pub mod template;

use crate::exit_with_message;
use crate::parse::ast::*;
use crate::parse::context::Context;

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
                    context.scopes.add_var_to_scope(param_name.clone(), param_type.clone());
                }

                for statement in statements {
                    validate_statement(statement, context);
                }

                context.scopes.pop_scope();
            }

            // TODO: 'self' parameter should be marked as 'inout'
            Item::Implementation { struct_name, functions  } => {
                context.validate_type(struct_name);

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
                                context.scopes.add_var_to_scope(param_name.clone(), param_type.clone());
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

            context.scopes.add_var_to_scope(ident.clone(), checked_type.unwrap());
        }

        Statement::LetConstructor { ident, constructor } => {
            context.scopes.add_var_to_scope(ident.clone(), constructor.ty.clone());
            
            for (_ident, field) in &mut constructor.fields {
                validate_expression(field, context);
            }
            
            // Order the fields and fill in defaults
            constructor.fields = context.generate_constructor(&constructor.ty, constructor.fields.clone());
        }

        Statement::Assignment { lhs, op, expression } => {
            let mut lhs_type = "".to_owned();

            validate_expression(lhs, context);
            
            // lhs is ident
            if let Expression::Identifier(ident) = lhs {
                if !context.scopes.is_var_in_scope(ident) {
                    exit_with_message(format!("Error: No such variable in scope: '{}'", ident));
                }
                lhs_type = context.scopes.var_type(ident);
            }
            // lhs is member field
            else if let Expression::Binary {lhs, operator, rhs, ty} = lhs {
                if let BinaryOperator::Member = operator {
                    lhs_type = ty.clone();
                } else {
                    exit_with_message(format!("Error: Assignment left hand side must be an identifier or field"));
                }
            }
            

            // rhs
            validate_expression(expression, context);
            let expr_type = context.expression_type(expression);

            if !Context::castable(&expr_type, &lhs_type) {
                // TODO: These errors
                // if is_member {
                //     exit_with_message(format!("Error: Field '{}' of struct '{}' cannot be assigned to incompatible type '{}'", , , &expr_type));
                // }
                // exit_with_message(format!("Error: Variable '{}' cannot be assigned to incompatible type '{}'", ident, &expr_type));
                exit_with_message(format!("Error: Invalid assignment statement. Cannot assign type '{}' to incompatible type '{}'", &lhs_type, &expr_type));
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
                context.scopes.add_var_to_scope(loop_var.clone(), "int".to_owned());
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
        Expression::FunctionCall { name, parameters, ty } => {
            let param_types = parameters.iter_mut().map(|expr| {
                validate_expression(expr, context);
                context.expression_type(expr)
            }).collect();
            
            let return_type = context.check_function_call(name, param_types);

            *ty = return_type;
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

        Expression::Binary { lhs, operator, rhs, ty } => {
            // Special case: member access including functions (does not exist in GLSL)
            if let BinaryOperator::Member = operator {
                // ident. (...)
                if let Expression::Identifier(ident) = lhs.as_mut() {
                    if !context.scopes.is_var_in_scope(ident) {
                        exit_with_message(format!("Error: Identifier '{}' was not found in the scope.", ident));
                    }

                    // ident.function()
                    if let Expression::FunctionCall { name, parameters, ty: func_ty } = rhs.as_mut() {
                        // Add self as first parameter
                        parameters.insert(0, Expression::Identifier(ident.clone()));
                        
                        let param_types = parameters.iter_mut().map(|expr| {
                            validate_expression(expr, context);
                            context.expression_type(expr)
                        }).collect();

                        *name = format!("__{}__{}", context.scopes.var_type(ident), name);
                        
                        *ty = context.check_function_call(&name, param_types);
                        *func_ty = ty.clone();
                    }
                    // ident.ident
                    else if let Expression::Identifier(name) = rhs.as_mut() {
                        *ty = context.struct_field_type(&context.scopes.var_type(ident), name);
                    }
                    // ident.expression
                    else {
                        exit_with_message("Error: Invalid use of dot operator".to_owned());
                    }

                // TODO: function().ident and function().function() ??
                } else {
                    exit_with_message(format!("Error: Dot operator can only be used on identifiers"));
                }

                return;
            }
            
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
                                // if Context::castable(&lhs_type, &type_name) {
                                if true {
                                    *ty = type_name.to_owned();
                                } else {
                                    exit_with_message(format!("Error: Cannot cast from type '{}' to '{}'", &lhs_type, &type_name));
                                }
                            } else {
                                exit_with_message(format!("Error: Cannot cast to non-primitive type, '{}'", &type_name));
                            }
                        }

                        _ => {
                            panic!("This cannot be reached");
                        }
                    }
                }

                BinaryOperator::Member => {
                    // Handled above (special case)
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

            Item::Implementation { struct_name, functions } => {
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