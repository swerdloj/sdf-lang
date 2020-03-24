use crate::parse::ast::*;
use crate::exit_with_message;

use std::collections::HashSet;

// TODO: Assign uniforms their default value (type checked)
pub fn translate_uniforms(uniforms: &HashSet<(String, String)>) -> String {
    let mut glsl = String::new();

    for (index, (name, ty)) in uniforms.iter().enumerate() {
        glsl.push_str(&format!("layout(location = {}) uniform {} {};\n", index, ty, name));
    }

    if uniforms.len() >= 1 {
        glsl.push('\n');
    }

    glsl
}

pub fn translate_outs(outs: &HashSet<(String, String)>) -> String {
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
pub fn translate_structure(name: &str, fields: &Vec<(String, String, Option<Expression>)>) -> String {
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

pub fn translate_function(name: &str, parameters: &Vec<(String, String)>, return_type: &str, statements: &Vec<Statement>) -> String {
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
        Statement::Continue => {
            glsl.push_str("continue");
        }
        Statement::Break => {
            glsl.push_str("break");
        }

        Statement::While { condition, block } => {
            glsl.push_str(&format!("while ({}) {{\n", translate_expression(condition)));

            for block_stmt in block {
                glsl.push_str(&format!("\t\t{}", translate_statement(block_stmt)));
            }

            glsl.push_str("\t}");
        }

        // TODO: Consider generating a while loop instead
        Statement::For { loop_var, from, to, block } => {
            // TODO: Require the expressions to be compile-time, then determine
            //       whether the loop should be > or < and ++ or --
            glsl.push_str(&format!("for (int {} = {}; {} < {}; ++{}) {{\n", loop_var, translate_expression(from), loop_var, translate_expression(to), loop_var));
            
            for block_stmt in block {
                glsl.push_str(&format!("\t\t{}", translate_statement(block_stmt)));
            }

            glsl.push_str("\t}");
        }

        Statement::Return { expression: expr } => {
            if let Some(ret_expr) = expr {
                glsl.push_str(&format!("return {}", translate_expression(ret_expr)));
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
                glsl.push_str(&format!(" = {}", translate_expression(assignment)));
            }

        }
        
        // Defaults and ordering will be handled while parsing
        Statement::LetConstructor { ident, constructor } => {
            let mut fields = String::new();

            for (_field_name, expr) in &constructor.fields {
                fields.push_str(&format!("{}, ", translate_expression(&expr)));
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
            }, translate_expression(&expr)));
        }
        
        Statement::Expression(expr) => {
            glsl.push_str(&translate_expression(expr));
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
        Expression::Literal(literal) => {
            match literal {
                Literal::Vector(vec) => {
                    let translate = |i: &IdentOrLiteral| {match i { IdentOrLiteral::Ident(ident) => ident.to_owned(),
                                                                                    IdentOrLiteral::Literal(lit) => translate_expression(&Expression::Literal(*lit.clone())).to_owned(), }};
                    match vec {
                        Vector::Vec2(f1, f2) => {
                            glsl.push_str(&format!("vec2({}, {})", translate(f1), translate(f2)));
                        }
                        Vector::Vec3(f1, f2, f3) => {
                            glsl.push_str(&format!("vec2({}, {}, {})", translate(f1), translate(f2), translate(f3)));
                        }
                        Vector::Vec4(f1, f2, f3, f4) => {
                            glsl.push_str(&format!("vec2({}, {}, {}, {})", translate(f1), translate(f2), translate(f3), translate(f4)));
                        }
                    }
                }
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

        Expression::Unary { operator, rhs, .. } => {
            match operator {
                UnaryOperator::Negate => {
                    glsl.push('-');
                }

                UnaryOperator::Not => {
                    glsl.push('!');
                }
            }

            glsl.push_str(&translate_expression(rhs));
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

        // TODO: Ideally, remove the trailing ";" generated by this particular statement
        // (although this is valid in GLSL)
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