use crate::parser::{Oprand, Type, AST};
use anyhow::Result;
use either::Either;
use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
    types::{BasicMetadataTypeEnum, BasicType},
    values::{BasicValue, BasicValueEnum, FunctionValue, PointerValue},
    IntPredicate, OptimizationLevel,
};
use std::{borrow::Borrow, collections::HashMap, path::Path};
use thiserror::Error;

#[derive(Error, Debug)]
enum CodeGenErr {
    #[error("Variable redefinition")]
    VariableRedefinition,
    #[error("Index of array should be integer")]
    IndexNotInt,
    #[error("Variable has not defined")]
    VariableNotDefined,
    #[error("Function redefinition")]
    FunctionRedefinition,
    #[error("Mismatched type")]
    MismatchedType,
    #[error("Mismatch type of Function's return type")]
    MismatchedTypeFunction,
    #[error("Function is not defined")]
    FunctionNotDefined,
    #[error("Expression has void type")]
    ExpressionVoidType,
}

pub struct CodeBuilder<'ctx> {
    /// A Context is a container for all LLVM entities including Modules.
    context: &'ctx Context,
    /// llvm `Module`: Each module directly contains a list of globals variables, a list of functions, a list of libraries (or other modules) this module depends on, a symbol table, and various data about the target's characteristics.
    module: Module<'ctx>,
    /// his provides a uniform API for creating instructions and inserting them into a basic block: either at the end of a BasicBlock, or at a specific iterator location in a block.
    builder: Builder<'ctx>,

    global_variables: HashMap<String, (Type, PointerValue<'ctx>)>,
    // 代表 作用域
    current_variables: Vec<HashMap<String, (Type, PointerValue<'ctx>)>>,
    current_function: Option<(Type, FunctionValue<'ctx>)>,
    global_functions: HashMap<String, (Type, FunctionValue<'ctx>)>,
}

impl<'ctx> CodeBuilder<'ctx> {
    pub fn new<T>(context: &'ctx Context, name: T, ast: &Vec<AST>) -> Result<Self>
    where
        T: Borrow<str>,
    {
        let builder = context.create_builder();
        let module = context.create_module(name.borrow());

        let mut codegen = Self {
            context,
            module,
            builder,
            global_variables: HashMap::new(),
            current_variables: Vec::new(),
            global_functions: HashMap::new(),
            current_function: None,
        };

        codegen.generate(ast)?;
        Ok(codegen)
    }
    /// Build llvm-ir assembly file
    pub fn build_llvmir(&self, path: &Path) {
        self.module.print_to_file(path).unwrap();
    }

    /// Build native assembly file in this machine
    pub fn build_asm(&self, path: &Path) {
        Target::initialize_native(&InitializationConfig::default())
            .expect("Failed to initialize native target");

        let triple = TargetMachine::get_default_triple();
        let cpu = TargetMachine::get_host_cpu_name().to_string();
        let features = TargetMachine::get_host_cpu_features().to_string();

        let target = Target::from_triple(&triple).unwrap();
        let machine = target
            .create_target_machine(
                &triple,
                &cpu,
                &features,
                OptimizationLevel::None,
                RelocMode::Default,
                CodeModel::Default,
            )
            .unwrap();

        // write assembly code
        machine
            .write_to_file(&self.module, FileType::Assembly, path)
            .unwrap();
    }

    fn generate(&mut self, ast: &Vec<AST>) -> Result<()> {
        for i in ast {
            match i {
                AST::FunctionDec(type_, name, params, body) => {
                    self.gen_function(type_, name, params, body)?
                }
                AST::VariableDec(type_, name) => self.gen_global_variable(type_, name)?,
                _ => panic!(),
            }
        }
        Ok(())
    }

    fn gen_global_variable(&mut self, type_: &Type, name: &str) -> Result<()> {
        if self.global_variables.contains_key(name) || self.global_functions.contains_key(name) {
            Err(CodeGenErr::VariableRedefinition)?
        }
        let v = self
            .module
            .add_global(type_.to_llvm_basic_type(&self.context), None, name);
        v.set_initializer(&self.context.i32_type().const_int(0, false));
        self.global_variables
            .insert(name.to_string(), (type_.clone(), v.as_pointer_value()));
        Ok(())
    }
    fn gen_function(
        &mut self,
        type_: &Type,
        name: &str,
        params: &Vec<(Type, String)>,
        body: &AST,
    ) -> Result<()> {
        if self.global_variables.contains_key(name) || self.global_functions.contains_key(name) {
            Err(CodeGenErr::FunctionRedefinition)?
        }
        let param_types: Vec<BasicMetadataTypeEnum<'ctx>> = params
            .iter()
            .map(|(param_type, _)| param_type.to_llvm_basic_metadata_type(self.context))
            .collect();
        let ty = match type_ {
            Type::Void => self.context.void_type().fn_type(&param_types[..], false),
            other => other
                .to_llvm_basic_type(self.context)
                .fn_type(&param_types[..], false),
        };

        let function = self.module.add_function(name, ty, None);
        self.current_function = Some((*type_, function));
        let basic_block = self
            .context
            .append_basic_block(self.current_function.unwrap().1, "entry");
        self.builder.position_at_end(basic_block);
        self.gen_block_stmt(body)?;
        if self.no_terminator() {
            self.builder.build_return(None);
        }

        self.global_functions
            .insert(name.to_string(), (*type_, function));
        Ok(())
    }

    fn gen_block_stmt(&mut self, ast: &AST) -> Result<()> {
        if let AST::BlockStmt(variables, statements) = ast {
            self.current_variables.push(HashMap::new());
            for var in variables {
                if let AST::VariableDec(type_, name) = var {
                    if self.current_variables.last().unwrap().contains_key(name) {
                        Err(CodeGenErr::VariableRedefinition)?
                    };
                    let v = self
                        .builder
                        .build_alloca(type_.to_llvm_basic_type(self.context), name);
                    self.current_variables
                        .last_mut()
                        .unwrap()
                        .insert(name.clone(), (type_.clone(), v));
                }
            }

            for stmt in statements {
                match stmt {
                    AST::BlockStmt(_, _) => self.gen_block_stmt(stmt)?,
                    AST::SelectionStmt(cond, then_stmt, else_stmt) => {
                        let comparison = self.gen_expression(cond)?.1.into_int_value();
                        let comparison = self.builder.build_int_truncate(
                            comparison,
                            self.context.bool_type(),
                            "condition",
                        );
                        let current_block = self.builder.get_insert_block().unwrap();

                        let then_block = self
                            .context
                            .insert_basic_block_after(current_block, "then_block");

                        let destination_block = self
                            .context
                            .insert_basic_block_after(then_block, "if_dest_block");
                        match else_stmt {
                            Some(else_stmt) => {
                                let else_block = self
                                    .context
                                    .prepend_basic_block(destination_block, "else_block");

                                self.builder
                                    .build_conditional_branch(comparison, then_block, else_block);
                                self.builder.position_at_end(then_block);
                                self.gen_block_stmt(then_stmt)?;
                                if self.no_terminator() {
                                    self.builder.build_unconditional_branch(destination_block);
                                }

                                self.builder.position_at_end(else_block);
                                self.gen_block_stmt(else_stmt)?;
                                if self.no_terminator() {
                                    self.builder.build_unconditional_branch(destination_block);
                                }
                            }
                            None => {
                                self.builder.build_conditional_branch(
                                    comparison,
                                    then_block,
                                    destination_block,
                                );
                                self.builder.position_at_end(then_block);
                                self.gen_block_stmt(then_stmt)?;
                                if self.no_terminator() {
                                    self.builder.build_unconditional_branch(destination_block);
                                }
                            }
                        };

                        self.builder.position_at_end(destination_block);
                    }
                    AST::IterationStmt(cond, loop_stmt) => {
                        let current_block = self.builder.get_insert_block().unwrap();
                        let loop_head = self
                            .context
                            .insert_basic_block_after(current_block, "loop_head");
                        self.builder.build_unconditional_branch(loop_head);

                        let loop_body = self
                            .context
                            .insert_basic_block_after(loop_head, "loop_body");
                        let destination_block = self
                            .context
                            .insert_basic_block_after(loop_body, "loop_dest_block");

                        self.builder.position_at_end(loop_head);
                        let comparison = self.gen_expression(cond)?.1.into_int_value();
                        let comparison = self.builder.build_int_truncate(
                            comparison,
                            self.context.bool_type(),
                            "condition",
                        );
                        self.builder.build_conditional_branch(
                            comparison,
                            loop_body,
                            destination_block,
                        );

                        self.builder.position_at_end(loop_body);
                        self.gen_block_stmt(loop_stmt)?;
                        self.builder.build_unconditional_branch(loop_head);

                        self.builder.position_at_end(destination_block);
                    }
                    AST::ReturnStmt(ret_value) => match ret_value {
                        Some(ast) => {
                            let (type_, value) = self.gen_expression(ast)?;
                            if type_ == self.current_function.unwrap().0 {
                                self.builder.build_return(Some(&value));
                            } else {
                                Err(CodeGenErr::MismatchedTypeFunction)?
                            }
                        }
                        None => {
                            self.builder.build_return(None);
                        }
                    },
                    AST::AssignmentExpr(var, expr) => {
                        self.gen_assinment_expr(var, expr)?;
                    }
                    AST::BinaryExpr(op, lhs, rhs) => {
                        self.gen_binary_expr(op, lhs, rhs)?;
                    }
                    AST::CallExpr(name, argments) => {
                        self.gen_function_call(name, argments)?;
                    }
                    _ => {}
                }
            }
            self.current_variables.pop();
        }
        Ok(())
    }

    fn gen_expression(&self, ast: &AST) -> Result<(Type, BasicValueEnum)> {
        match ast {
            AST::AssignmentExpr(var, expr) => self.gen_assinment_expr(var, expr),
            AST::BinaryExpr(op, lhs, rhs) => self.gen_binary_expr(op, lhs, rhs),
            AST::CallExpr(name, argments) => {
                // 在expression上下文中不应该返回void
                let r = self.gen_function_call(name, argments);
                if r.is_ok() && r.as_ref().unwrap().0 == Type::Void {
                    Err(CodeGenErr::ExpressionVoidType)?
                }
                r
            }
            AST::Variable(name, index) => {
                let (type_, ptr) = self.gen_variable(name, &index.as_ref().map(|x| x.as_ref()))?;
                let value = self.builder.build_load(ptr, "");
                Ok((type_, value))
            }
            AST::IntLiteral(value) => Ok((
                Type::Int,
                self.context
                    .i32_type()
                    .const_int(*value as u64, true)
                    .as_basic_value_enum(),
            )),
            _ => {
                panic!()
            }
        }
    }
    fn gen_binary_expr(
        &self,
        op: &Oprand,
        left: &AST,
        right: &AST,
    ) -> Result<(Type, BasicValueEnum)> {
        let (lhs, rhs) = (self.gen_expression(left)?.1, self.gen_expression(right)?.1);
        let lhs = match lhs {
            BasicValueEnum::IntValue(i) => i,
            BasicValueEnum::PointerValue(p) => {
                self.builder
                    .build_ptr_to_int(p, self.context.i32_type(), "")
            }
            _ => panic!(),
        };
        let rhs = match rhs {
            BasicValueEnum::IntValue(i) => i,
            BasicValueEnum::PointerValue(p) => {
                self.builder
                    .build_ptr_to_int(p, self.context.i32_type(), "")
            }
            _ => panic!(),
        };

        let value = match op {
            Oprand::Add => self.builder.build_int_add(lhs, rhs, ""),
            Oprand::Sub => self.builder.build_int_sub(lhs, rhs, ""),
            Oprand::Mul => self.builder.build_int_mul(lhs, rhs, ""),
            Oprand::Div => self.builder.build_int_signed_div(lhs, rhs, ""),
            Oprand::Ge => self
                .builder
                .build_int_compare(IntPredicate::SGE, lhs, rhs, ""),
            Oprand::Le => self
                .builder
                .build_int_compare(IntPredicate::SLE, lhs, rhs, ""),
            Oprand::Gt => self
                .builder
                .build_int_compare(IntPredicate::SGT, lhs, rhs, ""),
            Oprand::Lt => self
                .builder
                .build_int_compare(IntPredicate::SLT, lhs, rhs, ""),
            Oprand::Eq => self
                .builder
                .build_int_compare(IntPredicate::EQ, lhs, rhs, ""),
            Oprand::Ne => self
                .builder
                .build_int_compare(IntPredicate::NE, lhs, rhs, ""),
        }
        .as_basic_value_enum();

        Ok((Type::Int, value))
    }
    fn gen_function_call(&self, name: &str, argments: &Vec<AST>) -> Result<(Type, BasicValueEnum)> {
        let mut args = Vec::new();
        for argment in argments {
            let arg = self.gen_expression(argment)?.1;
            args.push(arg.into())
        }
        let function = self.global_functions.get(name);
        match function {
            Some((type_, function)) => {
                let return_value = self.builder.build_call(*function, &args[..], name);
                match return_value.try_as_basic_value() {
                    Either::Left(value) => {
                        if value.get_type() == type_.to_llvm_basic_type(self.context) {
                            return Ok((*type_, value));
                        } else {
                            Err(CodeGenErr::MismatchedType)?
                        }
                    }
                    Either::Right(_) => Ok((
                        Type::Void,
                        self.context
                            .i32_type()
                            .const_int(0, false)
                            .as_basic_value_enum(),
                    )),
                }
            }
            None => Err(CodeGenErr::FunctionNotDefined)?,
        }
    }
    fn gen_assinment_expr(&self, var: &AST, expr: &AST) -> Result<(Type, BasicValueEnum)> {
        if let AST::Variable(name, index) = var {
            let (type_left, ptr) = self.gen_variable(name, &index.as_ref().map(|x| x.as_ref()))?;
            let (type_right, value) = self.gen_expression(expr)?;
            if type_left == type_right {
                self.builder.build_store(ptr, value);
                Ok((type_left, value.as_basic_value_enum()))
            } else {
                Err(CodeGenErr::MismatchedType)?
            }
        } else {
            panic!()
        }
    }

    fn gen_variable(&self, name: &str, index: &Option<&AST>) -> Result<(Type, PointerValue)> {
        let (mut type_, mut ptr) = self.get_name_ptr(name)?;
        if let Some(index) = index {
            let (index_type, index) = self.gen_expression(index)?;
            if index_type == Type::Int {
                unsafe {
                    ptr = self.builder.build_gep(
                        ptr,
                        &[
                            self.context.i32_type().const_int(0, false),
                            index.into_int_value(),
                        ],
                        "",
                    );

                    // 因为只有整数这一个类型，否则应该继续判断左值的类型
                    type_ = Type::Int;
                }
            } else {
                Err(CodeGenErr::IndexNotInt)?
            }
        }
        Ok((type_, ptr))
    }

    // =============
    fn get_name_ptr(&self, name: &str) -> Result<(Type, PointerValue)> {
        for domain in self.current_variables.iter().rev() {
            if let Some(ptr) = domain.get(name) {
                return Ok(*ptr);
            }
        }
        if let Some(ptr) = self.global_variables.get(name) {
            return Ok(*ptr);
        }
        Err(CodeGenErr::VariableNotDefined)?
    }
    fn no_terminator(&self) -> bool {
        self.builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
    }
}

#[cfg(test)]
mod test_parse {
    use std::{fs::File, io::Read, path::Path};

    use inkwell::context::Context;

    use super::CodeBuilder;

    #[test]
    fn codegen_test() {
        let mut f = File::open("test/test.c").unwrap();
        let mut buf = String::new();
        f.read_to_string(&mut buf).unwrap();
        let ast = super::AST::parse(buf);
        let context = Context::create();
        let codegen = CodeBuilder::new(&context, "test", &ast).unwrap();
        codegen.build_asm(Path::new("test.asm"));
    }
}
