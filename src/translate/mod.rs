pub mod template;

use crate::exit_with_message;
use crate::parse::ast::*;
use crate::parse::context::Context;
use crate::parse::glsl_types;

// The parser generates an AST from the bottom up. This is an issue because expressions
// such as `if` will be parsed *after* the statements *within* the `if.
// Similarly, statement blocks are parsed after their statements, meaning scope is difficult
// to account for
//
// Note that nested items are translated recursively (for bottom-up analysis like expressions)
pub fn validate(ast: &mut AST, context: &mut Context) -> () {
    for item in ast {
        match item {
            Item::Struct { name, fields } => {
                context.declare_struct(name.clone(), fields.clone());
            }

            // TODO: Ensure that return statement has same type as function
            // TODO: Ensure that typed functions *have* a return statement
            Item::Function { name, parameters, return_type, statements } => {               
                context.declare_function(name.clone(), parameters.clone(), return_type.clone());

                context.scopes.push_scope("function");

                for (param_name, param_type) in parameters {
                    context.add_var_to_scope(param_name.clone(), param_type.clone());
                }

                for statement in statements {
                    validate_statement(statement, context);
                }

                context.scopes.pop_scope();
            }

            // TODO: 'self' parameter should be marked as 'inout'
            Item::Implementation { struct_name, functions  } => {
                context.validate_type(struct_name);
                context.declare_implementation(struct_name);

                if functions.len() == 0 {
                    exit_with_message(format!("Error: To implement '{}', at least one function is needed", struct_name));
                }

                for function in functions {
                    match function {
                        Item::Function { name, parameters, return_type, statements } => {
                            if parameters.len() > 0 {
                                parameters[0] = ("self".to_owned(), format!("{}", struct_name));
                            } else {
                                exit_with_message(format!("Error: Implementation function '{}.{}' must reference 'self'", struct_name, name));
                            }

                            // Memeber functions are represented like so in GLSL
                            *name = format!("__{}__{}", struct_name, name);
                            
                            context.declare_function(name.to_owned(), parameters.clone(), return_type.clone());
                            
                            context.scopes.push_scope("impl");

                            for (param_name, param_type) in parameters {
                                context.add_var_to_scope(param_name.clone(), param_type.clone());
                            }

                            for statement in statements {
                                validate_statement(statement, context);
                            }

                            context.scopes.pop_scope();
                        }
                        _ => {}
                    }
                }
            }

            Item::Scene { name, statements } => {
                // TODO: This
            }
        }
    }
}

fn validate_statement(statement: &mut Statement, context: &mut Context) {
    match statement {
        Statement::Continue | Statement::Break => {
            if !context.scopes.is_within_loop() {
                exit_with_message(format!("Error: 'continue' and 'break' are only valid within a loop (found in a {})", context.scopes.current_kind()));
            }
        }

        Statement::Let { ident, tag, ty, expression } => {
            if let Some(assignment) = expression {
                validate_expression(assignment, context);
            }
            
            // Tagged variables must have specified type and initial value
            if let Some(t) = tag {
                if expression.is_none() {
                    exit_with_message(format!("Semantic Error: Variable '{}' was tagged as '{:?}', but not initialized", ident, t));
                }

                if let Some(specified_type) = ty {   
                    match t {
                        Tag::Uniform => {
                            context.declare_uniform(ident.clone(), specified_type.clone())
                        }

                        _ => {
                            unimplemented!();
                        }
                    }
                } else {
                    exit_with_message(format!("Semantic Error: Variable '{}' was tagged as '{:?}', but its type was not specified", ident, t));
                }
            }

            let checked_type = if let Some(specified_type) = ty {
                context.validate_type(specified_type);
                // Check whether type assigned is compatible with user-specified
                if let Some(assignment) = expression {                    
                    let assigned_type = context.expression_type(assignment);
                    if !Context::castable(&assigned_type, &specified_type) {
                        exit_with_message(format!("Error: Variable '{}' was declared as a '{}', but assigned to an incompatible type: '{}'",
                                                   &ident, &specified_type, &assigned_type));
                        unreachable!();
                    }
                }
                Some(specified_type.clone())
            } else {
                // Make sure inferred type is valid (not void like a void function call)
                if let Some(assignment) = &expression {
                    let expr_type = context.expression_type(&assignment);
                    if expr_type != "void" {
                        Some(expr_type)
                    } else {
                        exit_with_message(format!("Error: Variable '{}' was assigned type 'void'.", &ident));
                        unreachable!();
                    }
                } else {
                    None
                }
            };

            *ty = checked_type.clone();

            context.add_var_to_scope(ident.clone(), checked_type.unwrap());
        }

        Statement::LetConstructor { ident, constructor } => {
            context.add_var_to_scope(ident.clone(), constructor.ty.clone());
            
            for (_ident, field) in &mut constructor.fields {
                validate_expression(field, context);
            }
            
            // Order the fields and fill in defaults
            constructor.fields = context.generate_constructor(&constructor.ty, constructor.fields.clone());
        }

        // The 'op' should not influence anything here
        Statement::Assignment { lhs, op, expression } => {
            let mut lhs_type = String::from("temp");

            match lhs {
                IdentOrMember::Ident(ident) => {
                    lhs_type = context.scopes.var_type(ident);
                }
                
                // lhs must be a series of identifiers and fields. No functions.
                IdentOrMember::Member(member) => {                   
                    for item in &member.path {
                        match item {
                            IdentOrFunction::Ident(ident) => {
                                // First item is a variable. The rest are fields.
                                if lhs_type == "temp" {
                                    lhs_type = context.scopes.var_type(ident);
                                } else {
                                    // Check if lhs is the field of a vec
                                    if glsl_types::vec::is_vec_constructor_or_type(&lhs_type) {
                                        // Ensure that swizzle is op-assignment valid (can be more than length 1)
                                        lhs_type = glsl_types::vec::validate_swizzle_for_assignment(&lhs_type, ident);
                                    } else {   
                                        lhs_type = context.struct_field_type(&lhs_type, ident);
                                    }
                                }
                            }

                            // TODO: Is this always true? Or are there cases where this would be valid?
                            IdentOrFunction::Function(func) => {
                                exit_with_message(format!("Error: Cannot assign to '.' operator with function call '{}'", func.name));
                            }
                        }
                    }
                }
            }

            // rhs
            validate_expression(expression, context);
            let expr_type = context.expression_type(expression);

            let result_type = match op {
                AssignmentOperator::Assign => {
                    expr_type
                }

                _ => {                    
                    // The result of (lhs op rhs) should be castable to the type of (lhs)
                    // This is useful for types like 'vec' where (lhs op rhs) is not always obvious
                    context.add_type(&lhs_type, &expr_type)
                    
                }
            };

            if !Context::castable(&result_type, &lhs_type) {
                exit_with_message(format!("Error: Invalid assignment statement. Cannot assign type '{}' to incompatible type '{}'", &lhs_type, &result_type));
            }
        }

        Statement::Return { expression } => {
            if let Some(expr) = expression {
                validate_expression(expr, context);
            }
        }

        Statement::For { loop_var, from, to, block } => {
            println!("WARNING: For loops are not fully implemented. Use a while loop instead.");
            
            context.scopes.push_scope("loop");
            
            let from_type = context.expression_type(from);
            let to_type = context.expression_type(to);

            if (from_type == "int" || from_type == "uint") && (to_type == "int" || to_type == "uint") {
                context.add_var_to_scope(loop_var.clone(), "int".to_owned());
            } else {
                exit_with_message("Error: For loops only support integers for now".to_owned());
            }

            for statement in block {
                validate_statement(statement, context);
            }

            context.scopes.pop_scope();
        }

        Statement::While { condition, block } => {
            if context.expression_type(condition) != "bool" {
                exit_with_message(format!("Error: While loop condition must be boolean."));
            }

            context.scopes.push_scope("loop");

            for statement in block {
                validate_statement(statement, context);
            }

            context.scopes.pop_scope();
        }

        Statement::Expression(expr) => {
            validate_expression(expr, context);
        }
    }
}

fn validate_expression(expression: &mut Expression, context: &mut Context) {
    match expression {
        Expression::FunctionCall(call) => {
            let param_types = call.parameters.iter_mut().map(|expr| {
                validate_expression(expr, context);
                context.expression_type(expr)
            }).collect();
            
            let return_type = context.check_function_call(&call.name, param_types);

            call.ty = return_type;
        }

        Expression::Unary { operator, rhs, ty } => {
            validate_expression(rhs, context);

            match operator {
                UnaryOperator::Negate => {
                    *ty = context.negate_type(&context.expression_type(rhs));
                }
                UnaryOperator::Not => {
                    if context.expression_type(rhs) != "bool" {
                        exit_with_message(format!("The binary not cannot be used on type '{}'", &ty));
                    }
                    *ty = "bool".to_owned();
                }
            }
        }

        Expression::Member(member) => {
            // First item
            let mut current_type = String::from("temp");
            let mut last_ident = String::new();

            let mut to_remove = Vec::new();

            // TODO: Re-order the path such that 
            //       [ident, f1(), b, f2()] = a.f1().b.c.f2() becomes
            //       f2( f1( a ).b.c )      <- function calls are moved to front
            for (index, item) in member.path.iter_mut().enumerate() {
                match item {
                    IdentOrFunction::Ident(ident) => {
                        if current_type == "temp" {
                            // First item must be a variable. Following would be fields.
                            current_type = context.scopes.var_type(ident);
                        } else {
                            // If vec type, follow swizzle rules
                            if glsl_types::vec::is_vec_constructor_or_type(&current_type) {
                                // Get the type of the swizzle
                                current_type = glsl_types::vec::validate_swizzle(&current_type, &ident);
                            } 
                            // Otherwise, it is just a normal field
                            else {
                                current_type = context.struct_field_type(&current_type, ident);
                            }
                        }
                        last_ident = ident.clone();
                    }

                    // Note that function calls cannot happen before identifiers
                    IdentOrFunction::Function(func) => {
                        if current_type == "temp" {
                            exit_with_message(format!("Error: Member methods must be accessed via the '.' operator: '{}'", func.name));
                        }
                        func.name = format!("__{}__{}", current_type, func.name);

                        // TODO: Also need to allow fields (not just single ident)
                        func.parameters.insert(0, Expression::Identifier(last_ident.clone()));
                        // ident was moved into the function call
                        to_remove.push(index - 1);

                        let param_types = func.parameters.iter_mut().map(|expr| {
                            validate_expression(expr, context);
                            context.expression_type(expr)
                        }).collect();

                        current_type = context.check_function_call(&func.name, param_types);
                        func.ty = current_type.clone();
                    }
                }
            }

            to_remove.into_iter().rev().map(|index| member.path.remove(index)).for_each(drop);

            member.ty = current_type;
        }

        Expression::Binary { lhs, operator, rhs, ty } => {            
            validate_expression(lhs, context);
            validate_expression(rhs, context);

            match operator {
                // TODO: Some operators like "&&" require lhs and rhs to both be boolean
                BinaryOperator::EqualTo | BinaryOperator::NotEqualTo | BinaryOperator::GreaterThanOrEqualTo 
                | BinaryOperator::LessThanOrEqualTo | BinaryOperator::GreaterThan | BinaryOperator::LessThan 
                | BinaryOperator::And | BinaryOperator::Or  => {}

                BinaryOperator::Multiply | BinaryOperator::Divide => {
                    let actual_type = context.multiply_type(
                        &context.expression_type(lhs),
                        &context.expression_type(rhs)
                    );

                    *ty = actual_type;
                }

                BinaryOperator::Plus | BinaryOperator::Minus => {
                    let actual_type = context.add_type(
                        &context.expression_type(lhs),
                        &context.expression_type(rhs)
                    );

                    *ty = actual_type;
                }

                BinaryOperator::Cast => {
                    let lhs_type = context.expression_type(&lhs);
                    // let rhs_type = context.expression_type(&rhs);

                    match rhs.as_ref() {
                        Expression::Identifier(type_name) => {
                            if context.is_primitive(&type_name) {
                                // TODO: Is this correct? Always required for narrowing conversions anyway
                                if Context::narrow_castable(&lhs_type, &type_name) {
                                // if true {
                                    *ty = type_name.to_owned();
                                } else {
                                    exit_with_message(format!("Error: Cannot cast from type '{}' to '{}'", &lhs_type, &type_name));
                                }
                            } else {
                                exit_with_message(format!("Error: Cannot cast to non-primitive type, '{}'", &type_name));
                            }
                        }

                        _ => {
                            exit_with_message(format!("Can only cast to type name, not an expression"));
                        }
                    }
                }
            }
        }

        // TODO: `If` is currently only treated as a statement. 
        //       Implement typing and translation for expression usage.
        Expression::If { expression, if_block, else_block, else_if_block, ty } => {
            validate_expression(expression, context);

            context.scopes.push_scope("if");
            for statement in if_block {
                validate_statement(statement, context);
            }
            context.scopes.pop_scope();
            
            if let Some(block_statements) = else_block {
                context.scopes.push_scope("if");
                for statement in block_statements {
                    validate_statement(statement, context);
                }
                context.scopes.pop_scope();
            }

            if let Some(else_if_expr) = else_if_block {
                validate_expression(else_if_expr, context);
            }

            // Condition must be type "bool"
            let expr_type = context.expression_type(expression);
            if expr_type != "bool" {
                exit_with_message(format!("Error: 'If' condition must be of type 'bool', but got '{}'", expr_type));
            }

            // TODO: Expression type check and assignment
        }

        Expression::Literal(_lit) => {
            // Nothing to do here
        }

        Expression::Identifier(ident) => {
            if !context.is_primitive(ident) && !context.scopes.is_var_in_scope(ident) {
                exit_with_message(format!("Error: Identifier '{}' not found in scope", ident));
            }
        }
    }
}

pub fn translate(ast: &AST, context: &Context) -> String {
    use template::*;

    // Unfortunately, GLSL requires functions to be declared in order of use
    // sdf-lang can compensate for this by forward declaring all functions
    // OR sdf-lang can also require forced ordering

    let mut glsl = String::new();

    // TODO: Allow user to specify version
    glsl.push_str("#version 450 core\n\n");

    glsl.push_str(&translate_uniforms(context.uniforms()));
    glsl.push_str(&translate_outs(context.outs()));

    // TODO: Allow let statements at global scope for global variables
    for item in ast {
        // `Item`s always have global scopes
        match item {
            Item::Struct { name, fields } => {
                glsl.push_str(&translate_structure(name, fields));
            }

            Item::Function { name, parameters, return_type, statements } => {
                // TODO: Body statements
                glsl.push_str(&translate_function(name, parameters, &return_type, statements));
            }

            Item::Scene { name, statements } => {
                glsl.push_str(&translate_scene(name, statements));
            }

            Item::Implementation { struct_name: _, functions } => {
                for function in functions {
                    match function {
                        Item::Function { name, parameters, return_type, statements } => {
                            glsl.push_str(&translate_function(name, parameters, &return_type, statements));
                        }

                        _ => {}
                    }
                }
            }
        }
    }

    glsl
}