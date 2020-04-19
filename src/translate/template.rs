use crate::parse::ast::*;

use std::collections::HashSet;

// TODO: Assign uniforms their default value (type checked)
pub fn translate_uniforms(uniforms: &HashSet<(String, TypeSpecifier)>) -> String {
    let mut glsl = String::new();

    for (index, (name, ty)) in uniforms.iter().enumerate() {
        glsl.push_str(&format!("layout(location = {}) uniform {};\n", index, translate_type_specifier(Some(name), &ty)));
    }

    if uniforms.len() >= 1 {
        glsl.push('\n');
    }

    glsl
}

pub fn translate_outs(outs: &HashSet<(String, TypeSpecifier)>) -> String {
    let mut glsl = String::new();

    for (index, (name, ty)) in outs.iter().enumerate() {
        glsl.push_str(&format!("layout(location = {}) out {};\n", index, translate_type_specifier(Some(name), &ty)));
    }

    if outs.len() >= 1 {
        glsl.push('\n');
    }

    glsl
}

pub fn translate_type_specifier(variable: Option<&str>, t: &TypeSpecifier) -> String {
    let mut glsl = String::new();

    glsl.push_str(t.type_name());

    if let Some(ref var) = variable {
        glsl.push(' ');
        glsl.push_str(var);
    }

    match t {
        TypeSpecifier::Identifier(_ident) => {},
        TypeSpecifier::Array { ty: _, size } => glsl.push_str(&format!("[{}]", size)),
    }

    glsl
}

pub fn translate_const(constant: &ConstDeclaration) -> String {
    let (ty, ident) = match &constant.ty {
        TypeSpecifier::Identifier(id) => {
            (id.clone(), constant.ident.clone())
        },
        TypeSpecifier::Array { ty, size } => {
            (ty.clone(), format!("{}[{}]", &constant.ident, size))
        }
    };
    //       const vec4 vert[x] = ...
    format!("const {} {} = {};\n\n", ty, ident, translate_expression(&constant.value.expression))
}

// Note that GLSL does not support struct defaults
pub fn translate_structure(name: &str, fields: &Vec<(String, TypeSpecifier, Option<Expression>)>) -> String {
    let mut glsl = String::new();

    glsl.push_str(&format!("struct {} {{\n", name));

    for (field, ty, _defaults) in fields {
        glsl.push_str(&format!("\t{} {};\n", ty, field));
    }

    // Remove trailing "\n"
    glsl.pop();

    glsl.push_str("\n};\n\n");

    glsl
}

pub fn translate_function(name: &str, parameters: &Vec<(Option<FuncParamQualifier>, String, TypeSpecifier)>, return_type: &TypeSpecifier, statements: &Vec<Statement>) -> String {
    let mut glsl = String::new();

    let mut param_string = String::new();
    for (qualifier, param_name, param_type) in parameters {
        if let Some(qual) = qualifier {
            match qual {
                FuncParamQualifier::In => param_string.push_str("in "),
                FuncParamQualifier::Out => param_string.push_str("out "),
                FuncParamQualifier::InOut => param_string.push_str("inout "),
            }
        }
        param_string.push_str(&format!("{}, ", translate_type_specifier(Some(param_name), param_type)));
    }

    // Remove trailing ", "
    param_string.pop();
    param_string.pop();

    glsl.push_str(&format!("{} {}({}) {{\n", translate_type_specifier(None, return_type), name, param_string));

    for nested_statement in statements {
        // Tagged variables are placed in global scope (required by GLSL)
        match nested_statement {
            Statement::Let { tag, .. } => {
                if let Some(_tag) = tag {
                    continue;
                }
            }
            _ => {},
        }

        glsl.push_str(&format!("\t{}", translate_statement(nested_statement)));
    }

    glsl.push_str("}\n\n");

    glsl
}

// TODO: This
// 
//       Scenes will need special scope treatment, as they introduce
//       SDF functions and types
pub fn translate_scene(name: &str, statements: &Vec<Statement>) -> String {
    let mut glsl = String::new();
    
    glsl.push_str(&format!("RayResult __scene__{}(vec3 point) {{\n", name));
    glsl.push_str("\tfloat dist; uint hit;");
    
    glsl.push_str("\n}\n\n");
    
    glsl
}

pub fn translate_statement(statement: &Statement) -> String {
    let mut glsl = String::new();

    match statement {
        Statement::Constant(constant) => {
            glsl.push_str(&translate_const(constant));

            // Remove the ";\n\n" added in translate_const
            glsl.pop();
            glsl.pop();
            glsl.pop();
        }

        Statement::Continue(_span) => {
            glsl.push_str("continue");
        }
        Statement::Break(_span) => {
            glsl.push_str("break");
        }

        Statement::While { condition, block, do_while } => {
            if *do_while {
                glsl.push_str("do {\n");
            } else {   
                glsl.push_str(&format!("while ({}) {{\n", translate_expression(&condition.expression)));
            }

            for block_stmt in block {
                glsl.push_str(&format!("\t\t{}", translate_statement(block_stmt)));
            }

            glsl.push_str("\t}");

            if *do_while {
                glsl.push_str(&format!(" while ({})", translate_expression(&condition.expression)));
            }
        }

        // TODO: Consider generating a while loop instead
        Statement::For { loop_var, from, to, block } => {
            // TODO: Require the expressions to be compile-time, then determine
            //       whether the loop should be > or < and ++ or --
            glsl.push_str(&format!("for (int {} = {}; {} < {}; ++{}) {{\n", 
                                            loop_var, translate_expression(&from.expression), 
                                            loop_var, translate_expression(&to.expression), loop_var));
            
            for block_stmt in block {
                glsl.push_str(&format!("\t\t{}", translate_statement(block_stmt)));
            }

            glsl.push_str("\t}");
        }

        Statement::Return { expression: expr } => {
            if let Some(ret_expr) = expr {
                glsl.push_str(&format!("return {}", translate_expression(&ret_expr.expression)));
            } else {
                glsl.push_str(&format!("return"));
            }
        }

        // TODO: Tagged variables should not be re-included here (handled elsewhere for global scope)
        Statement::Let { ident, ty, expression: expr, .. } => {           
            if ty.is_none() {
                // TODO: This error will eventually serve no purpose (after validation, all types are defined)
                panic!(format!("Error: The type of '{}' could not be determined. Consider annotating the type.", ident));
            }

            // FIXME: A lot of confusing shadowing happens here
            let (ty, ident) = match ty.as_ref().unwrap() {
                TypeSpecifier::Identifier(id) => {
                    (id.clone(), ident.clone())
                },
                TypeSpecifier::Array { ty, size } => {
                    (ty.clone(), format!("{}[{}]", ident, size))
                }
            };

            glsl.push_str(&format!("{} {}", ty, ident));

            if let Some(assignment) = expr {
                glsl.push_str(&format!(" = {}", translate_expression(&assignment.expression)));
            }
        }
        
        // Defaults and ordering will be handled while parsing
        Statement::LetConstructor { ident, constructor } => {
            let mut fields = String::new();

            for (_field_name, expr) in &constructor.fields {
                fields.push_str(&format!("{}, ", translate_expression(&expr.expression)));
            }

            // Remove trailing ", "
            fields.pop();
            fields.pop();
            
            glsl.push_str(&format!("{} {} = {}({})", constructor.ty, ident, constructor.ty, fields));
        }
        
        Statement::Assignment { lhs, op, expression: expr } => {
            let mut left = String::new();
            match lhs {
                IdentOrMember::Ident(ident) => left.push_str(ident),
                IdentOrMember::Member(member) => {
                    for item in &member.path {
                        if let IdentOrFunction::Ident(id) = item {
                            left.push_str(&format!("{}.", id));
                        }
                    }

                    // Remove trailing "."
                    left.pop();
                },
            };

            glsl.push_str(&format!("{} {} {}", left, match op {
                AssignmentOperator::Assign => "=",
                AssignmentOperator::AddAssign => "+=",
                AssignmentOperator::SubtractAssign => "-=",
                AssignmentOperator::MultiplyAssign => "*=",
                AssignmentOperator::DivideAssign => "/=",
            }, translate_expression(&expr.expression)));
        }
        
        Statement::Expression{expression, span: _} => {
            glsl.push_str(&translate_expression(expression));
        }
    }

    // Don't place ';' after statement blocks
    if let Some(last) = glsl.pop() {
        if last == '}' {
            glsl.push_str("}\n");
        } else {
            glsl.push_str(&format!("{};\n", last));
        }
    }

    glsl
}

pub fn translate_expression(expr: &Expression) -> String {
    let mut glsl = String::new();
    
    match expr {
        Expression::ArrayConstructor { expressions, ty } => {
            glsl.push_str(&format!("{}[](", ty));
            for item in expressions {
                glsl.push_str(&translate_expression(item));
                glsl.push_str(", ");
            }
            // Remove trailing ", "
            glsl.pop();
            glsl.pop();
            glsl.push(')');
        }

        Expression::Parenthesized(pexpr) => {
            glsl.push('(');
            glsl.push_str(&translate_expression(pexpr));
            glsl.push(')');
        }

        Expression::Literal(literal) => {
            match literal {
                Literal::Float(f) => {
                    // Rust prints "#.0f32" as "#" which is an int in glsl
                    let mut float = f.to_string();
                    if !float.contains(".") {
                        float.push('.');
                    }
                    glsl.push_str(&float);
                }
                Literal::Double(d) => {
                    glsl.push_str(&d.to_string());
                }
                Literal::Int(i) => {
                    glsl.push_str(&i.to_string());
                }
                Literal::UInt(u) => {
                    glsl.push_str(&u.to_string());
                }
                Literal::Bool(b) => {
                    glsl.push_str(&b.to_string());
                }
            }
        }

        Expression::Identifier(id) => {
            glsl.push_str(&id);
        }

        Expression::Binary { lhs, operator, rhs, .. } => {
            glsl.push_str(&match operator {
                BinaryOperator::Plus => {
                    format!("{} + {}", translate_expression(lhs), translate_expression(rhs))
                },
                BinaryOperator::Minus => {
                    format!("{} - {}", translate_expression(lhs), translate_expression(rhs))
                },
                BinaryOperator::Multiply => {
                    format!("{} * {}", translate_expression(lhs), translate_expression(rhs))
                },
                BinaryOperator::Divide => {
                    format!("{} / {}", translate_expression(lhs), translate_expression(rhs))
                },
                BinaryOperator::EqualTo => {
                    format!("{} == {}", translate_expression(lhs), translate_expression(rhs))
                },
                BinaryOperator::NotEqualTo => {
                    format!("{} != {}", translate_expression(lhs), translate_expression(rhs))
                },
                BinaryOperator::GreaterThan => {
                    format!("{} > {}", translate_expression(lhs), translate_expression(rhs))
                },
                BinaryOperator::LessThan => {
                    format!("{} < {}", translate_expression(lhs), translate_expression(rhs))
                },
                BinaryOperator::GreaterThanOrEqualTo => {
                    format!("{} >= {}", translate_expression(lhs), translate_expression(rhs))
                },
                BinaryOperator::LessThanOrEqualTo => {
                    format!("{} <= {}", translate_expression(lhs), translate_expression(rhs))
                },
                BinaryOperator::And => {
                    format!("{} && {}", translate_expression(lhs), translate_expression(rhs))
                },
                BinaryOperator::Or => {
                    format!("{} || {}", translate_expression(lhs), translate_expression(rhs))
                },

                BinaryOperator::Cast => {
                    format!("{}({})", translate_expression(rhs), translate_expression(lhs))
                }
            });
        }

        // TODO: When validating, should re-order these so any chained combination
        //       can translate simply
        Expression::Member(member) => {
            for item in &member.path {
                match item {
                    IdentOrFunction::Ident(ident) => {
                        glsl.push_str(ident);
                    }
                    
                    IdentOrFunction::Function(func) => {
                        glsl.push_str(&translate_expression(&Expression::FunctionCall(func.clone())));
                    }
                }
                glsl.push('.');
            }

            // Remove trailing "."
            glsl.pop();
        }

        Expression::Unary { operator, expr, .. } => {
            match operator {
                UnaryOperator::Index(index_expr) => {
                    glsl.push_str(&format!("{}[{}]", translate_expression(expr), translate_expression(index_expr)));
                }

                UnaryOperator::Negate => {
                    glsl.push_str(&format!("-{}", translate_expression(expr)));
                }

                UnaryOperator::Not => {
                    glsl.push_str(&format!("!{}", translate_expression(expr)));
                }
            }
        }

        Expression::FunctionApply(apply) => {
            glsl.push_str(&format!("{}(", apply.name));

            let mut current = 0;
            let mut parenthesis = 0;
            for expr in &apply.parameters {
                
                if apply.parameters.len() - current <= apply.func_parameters {
                    glsl.push_str(&format!("{}, ", translate_expression(expr)));
                } else {
                    glsl.push_str(&format!("{}, {}(", translate_expression(expr), apply.name));
                    parenthesis += 1;
                }

                current += 1;
            }

            // Remove trailing ", "
            glsl.pop();
            glsl.pop();

            // Close the opened parenthesis
            for _ in 0..=parenthesis {
                glsl.push(')');
            }
        }

        Expression::FunctionCall(call) => {
            let mut params = String::new();
            for subexpr in &call.parameters {
                params.push_str(&format!("{}, ", translate_expression(subexpr)));
            }

            // Remove traling ", "
            params.pop();
            params.pop();
            
            glsl.push_str(&format!("{}({})", call.name, params));
        }

        // TODO: Nested indentation is off
        Expression::If { expression: expr, if_block, else_block, else_if_block, ty } => {
            glsl.push_str(&format!("if ({}) {{\n", translate_expression(expr)));
            
            for stmt in if_block {   
                glsl.push_str(&format!("\t\t{}", translate_statement(stmt)));
            }

            if let Some(else_satements) = else_block {
                glsl.push_str("\t} else {\n");
                for stmt in else_satements {
                    glsl.push_str(&format!("\t\t{}", translate_statement(stmt)));
                }
            } else if let Some(else_if_statements) = else_if_block {
                glsl.push_str(&format!("\t}} else {}\n", translate_expression(else_if_statements)));
                
                // Remove trailing "\n\t}"
                glsl.pop();
                glsl.pop();
                glsl.pop();
            }

            glsl.push_str("\t}");
        }
    }

    glsl
}