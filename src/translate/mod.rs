pub mod template;

use crate::parse::ast::*;
use crate::parse::context::{Context, ScopeType};
use crate::parse::glsl;

// The parser generates an AST from the bottom up. This is an issue because expressions
// such as `if` will be parsed *after* the statements *within* the `if`.
// Similarly, statement blocks are parsed after their statements, meaning scope is difficult
// to account for.
//
// Note that nested items are translated recursively (for bottom-up type analysis like expressions)
pub fn validate(ast: &mut AST, input: &crate::parse::Input) -> Result<Context, String> {
    let mut context = Context::new(&input.shader_type);

    for item in ast {
        match item {
            Item::Constant(constant) => {
                let span = input.evaluate_span(constant.value.span);

                context.validate_type(&constant.ty).map_err(|e|
                    format!("{}\n{}", span, e)
                )?;
                let rhs_type = &context.expression_type(&constant.value.expression).map_err(|e|
                    format!("{}\n{}", span, e)
                )?;
                let castable = glsl::castable(rhs_type, &constant.ty).map_err(|e|
                    format!("{}\n{}", span, e)
                )?;

                if castable {
                    // This will be pushed to the global scope by default (no need to push/pop scope)
                    context.add_var_to_scope(constant.ident.clone(), constant.ty.clone(), true)?;
                } else {
                    return Err(format!("{}\nCannot assign the constant '{}' of type '{}' to incompatible type '{}'", span, constant.ident, constant.ty, rhs_type));
                }
            }

            Item::Struct { name, fields } => {
                context.declare_struct(name.clone(), fields.clone())?;
            }

            // TODO: Ensure that return statement has same type as function
            // TODO: Ensure that typed functions *have* a return statement
            Item::Function { name, parameters, return_type, statements } => {               
                context.declare_function(name.clone(), parameters.clone(), return_type.clone())?;

                context.scopes.push_scope(ScopeType::Function{ return_type: return_type.clone() });

                for (_param_qual, param_name, param_type) in parameters {
                    context.add_var_to_scope(param_name.clone(), param_type.clone(), false)?;
                }

                for statement in statements {
                    validate_statement(statement, &mut context, input)?;
                }

                context.scopes.pop_scope();
            }

            Item::Implementation { struct_name, functions  } => {
                context.validate_type(struct_name)?;
                context.declare_implementation(struct_name)?;

                if functions.len() == 0 {
                    return Err(format!("To implement '{}', at least one function is needed", struct_name));
                }

                for function in functions {
                    match function {
                        Item::Function { name, parameters, return_type, statements } => {
                            if parameters.len() > 0 {
                                // TODO: If unspecified, "self" will be marked as "inout"
                                //       Otherwise, don't force "inout"
                                parameters[0] = (Some(FuncParamQualifier::InOut), "self".to_owned(), format!("{}", struct_name));
                            } else {
                                return Err(format!("Implementation function '{}.{}' must reference 'self'", struct_name, name));
                            }

                            // Memeber functions are represented like so in GLSL
                            *name = format!("__{}__{}", struct_name, name);
                            
                            context.declare_function(name.to_owned(), parameters.clone(), return_type.clone())?;
                            
                            context.scopes.push_scope(ScopeType::Function{ return_type: return_type.clone() });

                            for (_qual, param_name, param_type) in parameters {
                                context.add_var_to_scope(param_name.clone(), param_type.clone(), false)?;
                            }

                            for statement in statements {
                                validate_statement(statement, &mut context, input)?;
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

    Ok(context)
}

// TODO: Give statements their own span
fn validate_statement(statement: &mut Statement, context: &mut Context, input: &crate::parse::Input) -> Result<(), String> {
    match statement {
        Statement::Continue(span) | Statement::Break(span) => {
            if !context.scopes.is_within_loop() {
                return Err(format!("{}\n'continue' and 'break' are only valid within a loop (found in a {:?})", input.evaluate_span(*span), context.scopes.current_kind()));
            }
        }

        Statement::Let { ident, tag, ty, expression } => {                       
            if ident.starts_with("gl_") {
                return Err(format!("The prefix 'gl_' is reserved (used in '{}')", ident));
            }
            
            if let Some(assignment) = expression {
                validate_expression(&mut assignment.expression, context, input).map_err(|e|
                    format!("{}\n{}", input.evaluate_span(assignment.span), e)
                )?;

                // Special cases
                if let Expression::Literal(lit) = &mut assignment.expression {
                    if let Literal::Int(i) = lit {
                        if let Some(t) = ty {
                            if t == "uint" {
                                *lit = Literal::UInt(*i as u32);
                            }
                        }
                    }
                } else if context.expression_type(&mut assignment.expression)? == "uint" {
                    if let Some(t) = ty {
                        if t == "int" {
                            return Err(format!("{}\nCannot assign 'int' to 'uint' expression", input.evaluate_span(assignment.span)));
                        }
                    }
                }
            }
            
            // Tagged variables must have specified type and initial value
            if let Some(t) = tag {
                if expression.is_none() {
                    return Err(format!("Variable '{}' was tagged as '{:?}', but not initialized", ident, t));
                }

                if let Some(specified_type) = ty {   
                    match t {
                        Tag::Uniform => {
                            context.declare_uniform(ident.clone(), specified_type.clone())?
                        }

                        _ => {
                            unimplemented!();
                        }
                    }
                } else {
                    return Err(format!("Variable '{}' was tagged as '{:?}', but its type was not specified", ident, t));
                }
            }

            let checked_type = if let Some(specified_type) = ty {
                context.validate_type(specified_type)?;
                // Check whether type assigned is compatible with user-specified
                if let Some(assignment) = expression {                    
                    let assigned_type = context.expression_type(&mut assignment.expression).map_err(|e| 
                        format!("{}\n{}", input.evaluate_span(assignment.span), e)
                    )?;

                    let castable = glsl::castable(&assigned_type, &specified_type).map_err(|e| 
                        format!("{}\n{}", input.evaluate_span(assignment.span), e)
                    )?;

                    if !castable {
                        return Err(format!("{}\nVariable '{}' was declared as type '{}', but assigned to an incompatible type: '{}'",
                                                input.evaluate_span(assignment.span), &ident, &specified_type, &assigned_type));
                    }
                }
                Some(specified_type.clone())
            } else {
                // Make sure inferred type is valid (not void like a void function call)
                if let Some(assignment) = &expression {
                    let expr_type = context.expression_type(&assignment.expression).map_err(|e| 
                        format!("{}\n{}", input.evaluate_span(assignment.span), e)
                    )?;
                    if expr_type != "void" {
                        Some(expr_type)
                    } else {
                        return Err(format!("{}\nVariable '{}' was assigned type 'void'.", input.evaluate_span(assignment.span), &ident));
                    }
                } else {
                    None
                }
            };

            *ty = checked_type.clone();

            context.add_var_to_scope(ident.clone(), checked_type.unwrap(), false)?;
        }

        Statement::LetConstructor { ident, constructor } => {
            context.add_var_to_scope(ident.clone(), constructor.ty.clone(), false)?;
            
            for (_ident, field) in &mut constructor.fields {
                validate_expression(&mut field.expression, context, input)?;
            }
            
            // Order the fields and fill in defaults
            constructor.fields = context.generate_constructor(&constructor.ty, constructor.fields.clone()).map_err(|e|
                // FIXME: Temporary hack to get a span for the LetConstructor
                if let Some(a) = constructor.fields.get(0) {
                    format!("{}\n{}", input.evaluate_span(a.1.span), e)
                } else {
                    e
                }
            )?;
        }

        // The 'op' should not influence anything here
        Statement::Assignment { lhs, op, expression } => {
            let span = input.evaluate_span(expression.span);
            
            let mut lhs_type = String::from("temp");

            match lhs {
                IdentOrMember::Ident(ident) => {
                    let is_constant = context.scopes.is_var_constant(ident).map_err(|e|
                        format!("{}\n{}", input.evaluate_span(expression.span), e)
                    )?;
                    if is_constant {
                        return Err(format!("{}\nCannot assign to an identifier declared as constant", input.evaluate_span(expression.span)));
                    }

                    lhs_type = context.scopes.var_type(ident)?;
                }
                
                // lhs must be a series of identifiers and fields. No functions.
                IdentOrMember::Member(member) => {                   
                    for item in &member.path {
                        match item {
                            IdentOrFunction::Ident(ident) => {
                                // First item is a variable. The rest are fields.
                                if lhs_type == "temp" {
                                    lhs_type = context.scopes.var_type(ident)?;
                                } else {
                                    // Check if lhs is the field of a vec
                                    if glsl::vec::is_vec_constructor_or_type(&lhs_type) {
                                        // Ensure that swizzle is op-assignment valid (can be more than length 1)
                                        lhs_type = glsl::vec::validate_swizzle_for_assignment(&lhs_type, ident)?;
                                    } else {   
                                        lhs_type = context.struct_field_type(&lhs_type, ident)?;
                                    }
                                }
                            }

                            // TODO: Is this always true? Or are there cases where this would be valid?
                            IdentOrFunction::Function(func) => {
                                return Err(format!("{}\nCannot assign to '.' operator with function call '{}'", span, func.name));
                            }
                        }
                    }
                }
            }

            // rhs
            validate_expression(&mut expression.expression, context, input).map_err(|e| 
                format!("{}\n{}", span, e)
            )?;
            let expr_type = context.expression_type(&expression.expression).map_err(|e| 
                format!("{}\n{}", span, e)
            )?;

            let result_type = match op {
                AssignmentOperator::Assign => {
                    expr_type
                }

                _ => {                    
                    // The result of (lhs op rhs) should be castable to the type of (lhs)
                    // This is useful for types like 'vec' where (lhs op rhs) is not always obvious
                    context.add_type(&lhs_type, &expr_type).map_err(|e|
                        format!("{}\n{}", span, e)
                    )?
                }
            };

            let castable = glsl::castable(&result_type, &lhs_type).map_err(|e|
                format!("{}\n{}", span, e)
            )?;

            if !castable {
                return Err(format!("{}\nInvalid assignment statement. Cannot assign type '{}' to incompatible type '{}'", span, &lhs_type, &result_type));
            }
        }

        // TODO: Ensure non-void function always return
        // TODO: Ensure if statements always lead to eventual returns
        Statement::Return { expression } => {
            let expected_type = context.scopes.expected_return_type()?;            

            if let Some(expr) = expression {
                let span = input.evaluate_span(expr.span);

                validate_expression(&mut expr.expression, context, input).map_err(|e|
                    format!("{}\n{}", span, e)
                )?;

                let ty = context.expression_type(&expr.expression).map_err(|e|
                    format!("{}\n{}", span, e)
                )?;

                let castable = glsl::castable(&ty, &expected_type).map_err(|e| 
                    format!("{}\n{}", span, e)
                )?;

                if !castable {
                    return Err(format!("{}\nExpected return type of '{}', but got incompatible type '{}'", span, expected_type, ty));
                }
            } else {
                if expected_type != "void" {
                    return Err(format!("Expected a '{}' return type, but found none", expected_type));
                }
            }
        }

        Statement::For { loop_var, from, to, block } => {
            // println!("WARNING: For loops are not fully implemented. Use a while loop instead.");
            
            context.scopes.push_scope(ScopeType::Loop);
            
            let from_type = context.expression_type(&from.expression).map_err(|e|
                format!("{}\n{}", input.evaluate_span(from.span), e)
            )?;
            let to_type = context.expression_type(&to.expression).map_err(|e|
                format!("{}\n{}", input.evaluate_span(to.span), e)
            )?;

            if (from_type == "int" || from_type == "uint") && (to_type == "int" || to_type == "uint") {
                context.add_var_to_scope(loop_var.clone(), "int".to_owned(), false).map_err(|e| 
                    // FIXME: Using "from.span" is a (viable) hack. Should use the for's span when implemented
                    format!("{}\n{}", input.evaluate_span(from.span), e)
                )?;
            } else {
                return Err("For loops only support integers for now".to_owned());
            }

            for statement in block {
                validate_statement(statement, context, input)?;
            }

            context.scopes.pop_scope();
        }

        Statement::While { condition, block } => {
            let span = input.evaluate_span(condition.span);
            
            let expr_type = context.expression_type(&condition.expression).map_err(|e|
                format!("{}\n{}", span, e)
            )?;

            if expr_type != "bool" {
                return Err(format!("{}\nWhile loop condition must be boolean", span));
            }

            context.scopes.push_scope(ScopeType::Loop);

            for statement in block {
                validate_statement(statement, context, input)?;
            }

            context.scopes.pop_scope();
        }

        Statement::Expression{ expression, span} => {
            validate_expression(expression, context, input).map_err(|e|
                format!("{}\n{}", input.evaluate_span(*span), e)
            )?;
        }
    }

    Ok(())
}

fn validate_expression(expression: &mut Expression, context: &mut Context, input: &crate::parse::Input) -> Result<(), String> {
    match expression {
        Expression::Parenthesized(expr) => {
            validate_expression(expr.as_mut(), context, input)?;
        }

        // TODO: This only seems plausable if the function accepts exactly 2 parameters
        Expression::FunctionApply(apply) => {
            let mut param_types = Vec::new();
            for expr in apply.parameters.iter_mut() {
                validate_expression(expr, context, input)?;
                param_types.push(context.expression_type(expr)?);
            }
            
            let (num_params, return_type) = context.check_function_apply(&apply.name, param_types)?;

            apply.func_parameters = num_params;
            apply.ty = return_type;
        }

        Expression::FunctionCall(call) => {
            let mut param_types = Vec::new();
            for expr in call.parameters.iter_mut() {
                validate_expression(expr, context, input)?;
                param_types.push(context.expression_type(expr)?);
            }
            
            let return_type = context.check_function_call(&call.name, param_types)?;

            call.ty = return_type;
        }

        Expression::Unary { operator, rhs, ty } => {
            validate_expression(rhs, context, input)?;

            match operator {
                UnaryOperator::Negate => {
                    *ty = context.negate_type(&context.expression_type(rhs)?)?;
                }
                UnaryOperator::Not => {
                    if context.expression_type(rhs)? != "bool" {
                        return Err(format!("The binary not cannot be used on type '{}'", &ty));
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
                            current_type = context.scopes.var_type(ident)?;
                        } else {
                            // If vec type, follow swizzle rules
                            if glsl::vec::is_vec_constructor_or_type(&current_type) {
                                // Get the type of the swizzle
                                current_type = glsl::vec::validate_swizzle(&current_type, &ident)?;
                            } 
                            // Otherwise, it is just a normal field
                            else {
                                current_type = context.struct_field_type(&current_type, ident)?;
                            }
                        }
                        last_ident = ident.clone();
                    }

                    // Note that function calls cannot happen before identifiers
                    IdentOrFunction::Function(func) => {
                        if current_type == "temp" {
                            return Err(format!("Member methods must be accessed via the '.' operator: '{}'", func.name));
                        }
                        func.name = format!("__{}__{}", current_type, func.name);

                        // TODO: Also need to allow fields (not just single ident)
                        func.parameters.insert(0, Expression::Identifier(last_ident.clone()));
                        // ident was moved into the function call
                        to_remove.push(index - 1);

                        let mut param_types = Vec::new();
                        for expr in func.parameters.iter_mut() {
                            validate_expression(expr, context, input)?;
                            param_types.push(context.expression_type(expr)?);
                        }

                        current_type = context.check_function_call(&func.name, param_types)?;
                        func.ty = current_type.clone();
                    }
                }
            }

            to_remove.into_iter().rev().for_each(|index| {member.path.remove(index);});

            member.ty = current_type;
        }

        Expression::Binary { lhs, operator, rhs, ty } => {            
            validate_expression(lhs, context, input)?;
            validate_expression(rhs, context, input)?;

            match operator {
                // TODO: Some operators like "&&" require lhs and rhs to both be boolean
                BinaryOperator::EqualTo | BinaryOperator::NotEqualTo | BinaryOperator::GreaterThanOrEqualTo 
                | BinaryOperator::LessThanOrEqualTo | BinaryOperator::GreaterThan | BinaryOperator::LessThan 
                | BinaryOperator::And | BinaryOperator::Or  => {}

                BinaryOperator::Multiply | BinaryOperator::Divide => {
                    let actual_type = context.multiply_type(
                        &context.expression_type(lhs)?,
                        &context.expression_type(rhs)?
                    )?;

                    *ty = actual_type;
                }

                BinaryOperator::Plus | BinaryOperator::Minus => {
                    let actual_type = context.add_type(
                        &context.expression_type(lhs)?,
                        &context.expression_type(rhs)?
                    )?;

                    *ty = actual_type;
                }

                BinaryOperator::Cast => {
                    let lhs_type = context.expression_type(&lhs)?;
                    // let rhs_type = context.expression_type(&rhs);

                    match rhs.as_ref() {
                        Expression::Identifier(type_name) => {
                            if context.is_primitive(&type_name) {
                                // TODO: Is this correct? Always required for narrowing conversions anyway
                                if glsl::narrow_castable(&lhs_type, &type_name)? {
                                // if true {
                                    *ty = type_name.to_owned();
                                } else {
                                    return Err(format!("Cannot cast from type '{}' to '{}'", &lhs_type, &type_name));
                                }
                            } else {
                                return Err(format!("Cannot cast to non-primitive type, '{}'", &type_name));
                            }
                        }

                        _ => return Err("Can only cast to type name, not an expression".to_owned()),                    }
                }
            }
        }

        // TODO: `If` is currently only treated as a statement. 
        //       Implement typing and translation for expression usage.
        Expression::If { expression, if_block, else_block, else_if_block, ty } => {
            validate_expression(expression, context, input)?;

            context.scopes.push_scope(ScopeType::If);
            for statement in if_block {
                validate_statement(statement, context, input)?;
            }
            context.scopes.pop_scope();
            
            if let Some(block_statements) = else_block {
                context.scopes.push_scope(ScopeType::If);
                for statement in block_statements {
                    validate_statement(statement, context, input)?;
                }
                context.scopes.pop_scope();
            }

            if let Some(else_if_expr) = else_if_block {
                validate_expression(else_if_expr, context, input)?;
            }

            // Condition must be type "bool"
            let expr_type = context.expression_type(expression)?;
            if expr_type != "bool" {
                return Err(format!("'If' condition must be of type 'bool', but got '{}'", expr_type));
            }

            // TODO: Expression type check and assignment
        }

        Expression::Literal(_lit) => {
            // Nothing to do here
        }

        Expression::Identifier(ident) => {
            if !context.is_primitive(ident) && !context.scopes.is_var_in_scope(ident) {
                return Err(format!("Identifier '{}' not found in scope", ident));
            }
        }
    }

    Ok(())
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
            Item::Constant(constant) => {
                glsl.push_str(&translate_const(constant));
            }

            Item::Struct { name, fields } => {
                glsl.push_str(&translate_structure(name, fields));
            }

            Item::Function { name, parameters, return_type, statements } => {
                // TODO: Body statements
                glsl.push_str(&translate_function(name, parameters, &return_type, statements));
            }

            Item::Scene { name, statements } => {
                // glsl.push_str(&translate_scene(name, statements));
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