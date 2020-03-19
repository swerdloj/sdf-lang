use crate::parse::ast::*;
use crate::exit_with_message;


pub fn uniforms(uniforms: &std::collections::HashSet<(String, String)>) -> String {
    let mut glsl = String::new();

    for (index, (name, ty)) in uniforms.iter().enumerate() {
        glsl.push_str(&format!("layout(location = {}) uniform {} {};\n", index, ty, name));
    }

    if uniforms.len() >= 1 {
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
    if parameters.len() > 0 {
        param_string.pop();
        param_string.pop();
    }

    glsl.push_str(&format!("{} {}({}) {{\n", return_type, name, param_string));

    for nested_statement in statements {
        glsl.push_str(&format!("\t{}", statement(nested_statement)));
    }

    glsl.push_str("}\n\n");

    glsl
}

// TODO: This
pub fn scene(name: &str) -> String {
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

        Statement::Let { ident, tag, ty, expression: expr } => {
            if ty.is_none() {
                exit_with_message(format!("The type of '{}' could not be determined. Consider annotating the type.", ident));
            }

            glsl.push_str(&format!("{} {} ", &ty.as_ref().unwrap(), ident));

            if let Some(assignment) = expr {
                glsl.push_str(&format!("= {}", expression(assignment)));
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
            glsl.push_str(&expression(lhs));

            glsl.push_str(match operator {
                BinaryOperator::Plus => " + ",
                BinaryOperator::Minus => " - ",
                BinaryOperator::Multiply => " * ",
                BinaryOperator::Divide => " / ",
                BinaryOperator::EqualTo => " == ",
                BinaryOperator::NotEqualTo => " != ",
                BinaryOperator::GreaterThan => " > ",
                BinaryOperator::LessThan => " < ",
                BinaryOperator::GreaterThanOrEqualTo => " >= ",
                BinaryOperator::LessThanOrEqualTo => " <= ",
                BinaryOperator::And => " && ",
                BinaryOperator::Or => " || ",
            });

            glsl.push_str(&expression(rhs));
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
                params.push_str(&format!("{},", expression(subexpr)));
            }

            // Remove traling ","
            params.pop();
            
            glsl.push_str(&format!("{}({})", name, params));
        }
    }

    glsl
}