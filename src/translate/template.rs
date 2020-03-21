use crate::parse::ast::*;
use crate::exit_with_message;

use std::collections::HashSet;

// TODO: Assign uniforms their default value (type checked)
pub fn uniforms(uniforms: &HashSet<(String, String)>) -> String {
    let mut glsl = String::new();

    for (index, (name, ty)) in uniforms.iter().enumerate() {
        glsl.push_str(&format!("layout(location = {}) uniform {} {};\n", index, ty, name));
    }

    if uniforms.len() >= 1 {
        glsl.push('\n');
    }

    glsl
}

pub fn outs(outs: &HashSet<(String, String)>) -> String {
    let mut glsl = String::new();

    for (index, (name, ty)) in outs.iter().enumerate() {
        glsl.push_str(&format!("layout(location = {}) out {} {};\n", index, ty, name));
    }

    if outs.len() >= 1 {
        glsl.push('\n');
    }

    glsl
}

// Note that GLSL does not support struct defaults
pub fn structure(name: &str, fields: &Vec<(String, String, Option<Expression>)>) -> String {
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

pub fn function(name: &str, parameters: &Vec<(String, String)>, return_type: &str, statements: &Vec<Statement>) -> String {
    let mut glsl = String::new();

    let mut param_string = String::new();
    for (param_name, param_type) in parameters {
        param_string.push_str(&format!("{} {}, ", param_type, param_name));
    }

    // Remove trailing ", "
    param_string.pop();
    param_string.pop();

    glsl.push_str(&format!("{} {}({}) {{\n", return_type, name, param_string));

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

        glsl.push_str(&format!("\t{}", statement(nested_statement)));
    }

    glsl.push_str("}\n\n");

    glsl
}

// TODO: This
// 
//       Scenes will need special scope treatment, as they introduce
//       SDF functions and types
pub fn scene(name: &str, statements: &Vec<Statement>) -> String {
    let mut glsl = String::new();
    
    glsl.push_str(&format!("RayResult __scene__{}(vec3 point) {{\n", name));
    glsl.push_str("\tfloat dist; uint hit;");
    
    glsl.push_str("\n}\n\n");
    
    glsl
}

pub fn statement(statement: &Statement) -> String {
    let mut glsl = String::new();

    match statement {
        Statement::Return { expression: expr } => {
            if let Some(ret_expr) = expr {
                glsl.push_str(&format!("return {}", expression(ret_expr)));
            } else {
                glsl.push_str(&format!("return"));
            }
        }

        // TODO: Tagged variables should not be re-included here (handled elsewhere for global scope)
        Statement::Let { ident, ty, expression: expr, .. } => {           
            if ty.is_none() {
                exit_with_message(format!("Error: The type of '{}' could not be determined. Consider annotating the type.", ident));
            }

            glsl.push_str(&format!("{} {}", &ty.as_ref().unwrap(), ident));

            if let Some(assignment) = expr {
                glsl.push_str(&format!(" = {}", expression(assignment)));
            }

        }
        
        // Defaults and ordering will be handled while parsing
        Statement::LetConstructor { ident, constructor } => {
            let mut fields = String::new();

            for (_field_name, expr) in &constructor.fields {
                fields.push_str(&format!("{}, ", expression(&expr)));
            }

            // Remove trailing ", "
            fields.pop();
            fields.pop();
            
            glsl.push_str(&format!("{} {} = {}({})", constructor.ty, ident, constructor.ty, fields));
        }
        
        Statement::Assignment { ident, op, expression: expr } => {
            glsl.push_str(&format!("{} {} {}", ident, match op {
                AssignmentOperator::Assign => "=",
                AssignmentOperator::AddAssign => "+=",
                AssignmentOperator::SubtractAssign => "-=",
                AssignmentOperator::MultiplyAssign => "*=",
                AssignmentOperator::DivideAssign => "/=",
            }, expression(&expr)));
        }
        
        Statement::Expression(expr) => {
            glsl.push_str(&expression(expr));
        }
    }

    glsl.push_str(";\n");

    glsl
}

pub fn expression(expr: &Expression) -> String {
    let mut glsl = String::new();
    
    match expr {
        Expression::Literal(literal) => {
            match literal {
                Literal::Float(f) => {
                    glsl.push_str(&f.to_string());
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
                    format!("{} + {}", expression(lhs), expression(rhs))
                },
                BinaryOperator::Minus => {
                    format!("{} - {}", expression(lhs), expression(rhs))
                },
                BinaryOperator::Multiply => {
                    format!("{} * {}", expression(lhs), expression(rhs))
                },
                BinaryOperator::Divide => {
                    format!("{} / {}", expression(lhs), expression(rhs))
                },
                BinaryOperator::EqualTo => {
                    format!("{} == {}", expression(lhs), expression(rhs))
                },
                BinaryOperator::NotEqualTo => {
                    format!("{} != {}", expression(lhs), expression(rhs))
                },
                BinaryOperator::GreaterThan => {
                    format!("{} > {}", expression(lhs), expression(rhs))
                },
                BinaryOperator::LessThan => {
                    format!("{} < {}", expression(lhs), expression(rhs))
                },
                BinaryOperator::GreaterThanOrEqualTo => {
                    format!("{} >= {}", expression(lhs), expression(rhs))
                },
                BinaryOperator::LessThanOrEqualTo => {
                    format!("{} <= {}", expression(lhs), expression(rhs))
                },
                BinaryOperator::And => {
                    format!("{} && {}", expression(lhs), expression(rhs))
                },
                BinaryOperator::Or => {
                    format!("{} || {}", expression(lhs), expression(rhs))
                },

                BinaryOperator::Cast => {
                    format!("{}({})", expression(rhs), expression(lhs))
                }
                BinaryOperator::Member => {
                    format!("{}.{}", expression(lhs), expression(rhs))
                }
            });
        }

        Expression::Unary { operator, rhs, .. } => {
            match operator {
                UnaryOperator::Negate => {
                    glsl.push('-');
                }

                UnaryOperator::Not => {
                    glsl.push('!');
                }
            }

            glsl.push_str(&expression(rhs));
        }

        Expression::FunctionCall { name, parameters, .. } => {
            let mut params = String::new();
            for subexpr in parameters {
                params.push_str(&format!("{}, ", expression(subexpr)));
            }

            // Remove traling ", "
            params.pop();
            params.pop();
            
            glsl.push_str(&format!("{}({})", name, params));
        }

        // TODO: Ideally, remove the trailing ";" generated by this particular statement
        // (although this is valid in GLSL)
        // TODO: Nested indentation is off
        Expression::If { expression: expr, if_block, else_block, else_if_block, ty } => {
            glsl.push_str(&format!("if ({}) {{\n", expression(expr)));
            
            for stmt in if_block {   
                glsl.push_str(&format!("\t\t{}", statement(stmt)));
            }

            if let Some(else_satements) = else_block {
                glsl.push_str("\t} else {\n");
                for stmt in else_satements {
                    glsl.push_str(&format!("\t\t{}", statement(stmt)));
                }
            } else if let Some(else_if_statements) = else_if_block {
                glsl.push_str(&format!("\t}} else {}\n", expression(else_if_statements)));
                
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