use crate::commands::generate::model::{
    AbstractDataType, DataModelNode, FullyQualifiedTypeReference, NamespaceNode,
};
use crate::commands::generate::result::{invalid_abstract_data_type_error, CodeGenError};
use convert_case::{Case, Casing};
use itertools::Itertools;
use std::fmt::{Display, Formatter};

pub trait Language {
    /// Provides a file extension based on programming language
    fn file_extension() -> String;

    /// Returns string representation of programming language name
    fn name() -> String;

    /// Provides generated code's file name for given type `name` based on programming language standards
    /// e.g.
    ///     In Rust, this will return a string casing `name` to [Case::Snake].
    ///     In Java, this will return a string casing `name` to  [Case::UpperCamel]
    fn file_name_for_type(name: &str) -> String;

    /// Maps the given ISL type to a target type name
    /// Returns None when the given ISL type is `struct`, `list` or `sexp` as open-ended types are not supported currently.
    fn target_type(ion_schema_type: &IonSchemaType) -> Option<String>;

    /// Provides given target type as sequence
    /// e.g.
    ///     target_type = "Foo" returns "java.util.ArrayList<Foo>"
    ///     target_type = "Foo" returns "Vec<Foo>"
    #[allow(dead_code)]
    fn target_type_as_sequence(
        target_type: FullyQualifiedTypeReference,
    ) -> FullyQualifiedTypeReference;

    /// Returns true if the type `String` specified is provided by the target language implementation
    fn is_built_in_type(type_name: String) -> bool;

    /// Returns a fully qualified type reference name as per the programming language
    /// e.g. For a fully qualified type reference as below:
    ///   FullyQualifiedTypeReference {
    ///     type_name: vec!["org", "example", "Foo"],
    ///     parameters: vec![] // type ref with no parameters
    ///   }
    ///   In Java, `org.example.Foo`
    ///   In Rust, `org::example::Foo`
    #[allow(dead_code)]
    fn fully_qualified_type_ref(name: &FullyQualifiedTypeReference) -> String;

    /// Returns the template as string based on programming language
    /// e.g.
    ///     In Rust, Template::Struct -> "struct"
    ///     In Java, Template::Struct -> "class"
    fn template_name(template: &Template) -> String;

    /// Returns the namespace separator for programming language
    /// e.g. In Java, it returns "::"
    ///      In Rust, it returns "."
    fn namespace_separator() -> &'static str;

    /// Modifies the given namespace to add the given type to the namespace path.
    /// _Note:_ For Rust, it uses the `is_nested_type` field to only get modules in the path name until the leaf type is reached.
    ///    e.g. given a module as below:
    ///         ```
    ///         mod foo {
    ///             struct Foo { ... }
    ///             mod nested_type {
    ///                 struct NestedType { ... }
    ///             }
    ///         }
    ///         ```
    ///     To add `NestedType` into the namespace path, `is_nested_type` helps remove any prior types form the path and add this current type.
    ///     i.e. given namespace path as `foo::Foo`, it will first remove `Foo` and then add the current type as `foo::nested_type::NestedType`.
    fn add_type_to_namespace(
        is_nested_type: bool,
        type_name: &str,
        namespace: &mut Vec<NamespaceNode>,
    );

    /// Resets the namespace when code generation is complete for a single ISL type
    fn reset_namespace(namespace: &mut Vec<NamespaceNode>);

    /// Returns the `FullyQualifiedReference` that represents the target type as optional in the given programming language
    /// e.g. In Java, it will return "java.util.Optional<T>"
    ///     In Rust, it will return "Option<T>"
    fn target_type_as_optional(
        target_type: FullyQualifiedTypeReference,
    ) -> FullyQualifiedTypeReference;
}

pub struct JavaLanguage;

impl Language for JavaLanguage {
    fn file_extension() -> String {
        "java".to_string()
    }

    fn name() -> String {
        "java".to_string()
    }

    fn file_name_for_type(name: &str) -> String {
        name.to_case(Case::UpperCamel)
    }

    fn target_type(ion_schema_type: &IonSchemaType) -> Option<String> {
        use IonSchemaType::*;
        Some(
            match ion_schema_type {
                Int => "int",
                String | Symbol => "String",
                Float => "double",
                Bool => "boolean",
                Blob | Clob => "byte[]",
                List | SExp | Struct => return None,
                SchemaDefined(name) => name,
            }
            .to_string(),
        )
    }

    fn target_type_as_sequence(
        target_type: FullyQualifiedTypeReference,
    ) -> FullyQualifiedTypeReference {
        match JavaLanguage::wrapper_class_or_none(
            &target_type.string_representation::<JavaLanguage>(),
        ) {
            Some(wrapper_name) => FullyQualifiedTypeReference {
                type_name: vec![
                    NamespaceNode::Package("java".to_string()),
                    NamespaceNode::Package("util".to_string()),
                    NamespaceNode::Type("ArrayList".to_string()),
                ],
                parameters: vec![FullyQualifiedTypeReference {
                    type_name: vec![NamespaceNode::Type(wrapper_name)],
                    parameters: vec![],
                }],
            },
            None => FullyQualifiedTypeReference {
                type_name: vec![
                    NamespaceNode::Package("java".to_string()),
                    NamespaceNode::Package("util".to_string()),
                    NamespaceNode::Type("ArrayList".to_string()),
                ],
                parameters: vec![target_type],
            },
        }
    }

    fn is_built_in_type(type_name: String) -> bool {
        matches!(
            type_name.as_str(),
            "int" | "String" | "boolean" | "byte[]" | "double"
        )
    }

    fn fully_qualified_type_ref(name: &FullyQualifiedTypeReference) -> String {
        name.type_name.iter().map(|n| n.name()).join(".")
    }

    fn template_name(template: &Template) -> String {
        match template {
            Template::Struct => "class".to_string(),
            Template::Scalar => "scalar".to_string(),
            Template::Sequence => "sequence".to_string(),
            Template::Enum => "enum".to_string(),
        }
    }

    fn namespace_separator() -> &'static str {
        "."
    }

    fn add_type_to_namespace(
        _is_nested_type: bool,
        type_name: &str,
        namespace: &mut Vec<NamespaceNode>,
    ) {
        namespace.push(NamespaceNode::Type(type_name.to_case(Case::UpperCamel)))
    }

    fn reset_namespace(namespace: &mut Vec<NamespaceNode>) {
        // resets the namespace by removing current abstract dta type name
        namespace.pop();
    }

    fn target_type_as_optional(
        target_type: FullyQualifiedTypeReference,
    ) -> FullyQualifiedTypeReference {
        match JavaLanguage::wrapper_class_or_none(
            &target_type.string_representation::<JavaLanguage>(),
        ) {
            Some(wrapper_name) => FullyQualifiedTypeReference {
                type_name: vec![NamespaceNode::Type(wrapper_name)],
                parameters: vec![],
            },
            None => target_type,
        }
    }
}

impl JavaLanguage {
    /// Returns the wrapper class for the given primitive data type, otherwise returns None.
    fn wrapper_class_or_none(primitive_data_type: &str) -> Option<String> {
        match primitive_data_type {
            "int" => Some("Integer".to_string()),
            "boolean" => Some("Boolean".to_string()),
            "double" => Some("Double".to_string()),
            "long" => Some("Long".to_string()),
            _ => {
                // for any other non-primitive types return None
                None
            }
        }
    }

    /// Returns the wrapper class for the given primitive data type
    /// If `data_type` is a primitive, returns the boxed equivalent, otherwise returns `data_type`.
    pub fn wrapper_class(primitive_data_type: &str) -> String {
        match Self::wrapper_class_or_none(primitive_data_type) {
            None => primitive_data_type.to_string(),
            Some(wrapper_class) => wrapper_class,
        }
    }

    /// Returns the primitive data type for the given wrapper class, or `wrapper_class`
    /// if it does not have an equivalent primitive type.
    pub fn primitive_data_type(wrapper_class: &str) -> &str {
        match wrapper_class {
            "Integer" => "int",
            "Boolean" => "boolean",
            "Double" => "double",
            "Long" => "long",
            "Short" => "short",
            "Byte" => "byte",
            "Float" => "float",
            "Character" => "char",
            _ => wrapper_class,
        }
    }
}

impl Display for JavaLanguage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "java")
    }
}

pub struct RustLanguage;

impl Language for RustLanguage {
    fn file_extension() -> String {
        "rs".to_string()
    }

    fn name() -> String {
        "rust".to_string()
    }

    fn file_name_for_type(_name: &str) -> String {
        "ion_generated_code".to_string()
    }

    fn target_type(ion_schema_type: &IonSchemaType) -> Option<String> {
        use IonSchemaType::*;
        Some(
            match ion_schema_type {
                Int => "i64",
                String | Symbol => "String",
                Float => "f64",
                Bool => "bool",
                Blob | Clob => "Vec<u8>",
                List | SExp | Struct => return None,
                SchemaDefined(name) => name,
            }
            .to_string(),
        )
    }

    fn target_type_as_sequence(
        target_type: FullyQualifiedTypeReference,
    ) -> FullyQualifiedTypeReference {
        FullyQualifiedTypeReference {
            type_name: vec![NamespaceNode::Type("Vec".to_string())],
            parameters: vec![target_type],
        }
    }

    fn is_built_in_type(type_name: String) -> bool {
        matches!(
            type_name.as_str(),
            "i64" | "String" | "bool" | "Vec<u8>" | "f64"
        )
    }

    fn fully_qualified_type_ref(name: &FullyQualifiedTypeReference) -> String {
        name.type_name.iter().map(|n| n.name()).join("::")
    }

    fn template_name(template: &Template) -> String {
        match template {
            Template::Struct => "struct".to_string(),
            Template::Scalar => "scalar".to_string(),
            Template::Sequence => "sequence".to_string(),
            Template::Enum => {
                //TODO: Rust enums are not supported yet
                // The template `enum.templ` is just a placeholder
                "enum".to_string()
            }
        }
    }

    fn namespace_separator() -> &'static str {
        "::"
    }

    fn add_type_to_namespace(
        is_nested_type: bool,
        type_name: &str,
        namespace: &mut Vec<NamespaceNode>,
    ) {
        // e.g. For example there is a `NestedType` inside `Foo` struct. Rust code generation also generates similar modules for the generated structs.
        // ```rust
        // mod foo {
        //   struct Foo {
        //     ...
        //   }
        //   mod nested_type {
        //      struct NestedType {
        //        ...
        //      }
        //   }
        // }
        // ```
        if is_nested_type {
            if let Some(last_value) = namespace.last() {
                // Assume we have the current namespace as `foo::Foo`
                // then the following step will remove `Foo` from the path for nested type.
                // So that the final namespace path for `NestedType` will become `foo::nested_type::NestedType`
                if !matches!(last_value, NamespaceNode::Package(_)) {
                    // if the last value is not module name then pop the type name from namespace
                    namespace.pop(); // Remove the parent struct/enum
                }
            }
        }
        namespace.push(NamespaceNode::Package(type_name.to_case(Case::Snake))); // Add this type's module name to the namespace path
        namespace.push(NamespaceNode::Type(type_name.to_case(Case::UpperCamel)))
        // Add this type itself to the namespace path
    }

    fn reset_namespace(namespace: &mut Vec<NamespaceNode>) {
        // Resets the namespace by removing current abstract data type name and module name
        if let Some(last_value) = namespace.last() {
            // Check if it is a type then pop the type and module
            if matches!(last_value, NamespaceNode::Package(_)) {
                // if this is a module then only pop once for the module
                namespace.pop();
            } else if matches!(last_value, NamespaceNode::Type(_)) {
                namespace.pop();
                if !namespace.is_empty() {
                    namespace.pop();
                }
            }
        }
    }

    fn target_type_as_optional(
        target_type: FullyQualifiedTypeReference,
    ) -> FullyQualifiedTypeReference {
        // TODO: un-comment following block for optional support in Rust, once the templates are changes accordingly
        // FullyQualifiedTypeReference {
        //     type_name: vec!["Option".to_string()],
        //     parameters: vec![target_type],
        // }
        target_type
    }
}

impl Display for RustLanguage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "rust")
    }
}

/// Represents all the supported templates for code generation.
/// These templates will be used by [tera] templating engine to render the generated code with appropriate context value.
/// _Note: These template variants are based on Rust programming language.
/// [Template::name] provides an appropriate template file name based on given programming language._
///
/// [tera]: <https://docs.rs/tera/latest/tera/>
pub enum Template {
    Struct,   // Represents a template for a Rust struct or Java class with Ion struct value
    Sequence, // Represents a template for a Rust struct or Java class with Ion sequence value
    Scalar,   // Represents a template for a Rust struct or Java class with Ion scalar value
    Enum,     // Represents a template for a Rust or Java enum
}

impl TryFrom<&DataModelNode> for Template {
    type Error = CodeGenError;

    fn try_from(value: &DataModelNode) -> Result<Self, Self::Error> {
        if let Some(abstract_data_type) = &value.code_gen_type {
            match abstract_data_type {
                AbstractDataType::Scalar(_) | AbstractDataType::WrappedScalar(_) => {
                    Ok(Template::Scalar)
                }
                AbstractDataType::Sequence(_) | AbstractDataType::WrappedSequence(_) => {
                    Ok(Template::Sequence)
                }
                AbstractDataType::Structure(_) => Ok(Template::Struct),
                AbstractDataType::Enum(_) => Ok(Template::Enum),
            }
        } else {
            invalid_abstract_data_type_error(
                "Can not get a template without determining data model first.",
            )
        }
    }
}

/// Represents an Ion schema type which could either be one of the [built-int types] or a user defined type.
///
/// [built-in types]: `<https://amazon-ion.github.io/ion-schema/docs/isl-2-0/spec#built-in-types>`
// TODO: Add enum variants for missing built-in ISL types.
pub enum IonSchemaType {
    Int,
    String,
    Symbol,
    Float,
    Bool,
    Blob,
    Clob,
    SExp,
    List,
    Struct,
    SchemaDefined(String), // A user defined schema type
}

impl From<&str> for IonSchemaType {
    fn from(value: &str) -> Self {
        use IonSchemaType::*;
        match value {
            "int" => Int,
            "string" => String,
            "symbol" => Symbol,
            "float" => Float,
            "bool" => Bool,
            "blob" => Blob,
            "clob" => Clob,
            _ if &value[..1] == "$" => {
                unimplemented!("Built in types with nulls are not supported yet!")
            }
            "number" | "text" | "lob" | "document" | "nothing" => {
                unimplemented!("Complex types are not supported yet!")
            }
            "decimal" | "timestamp" => {
                unimplemented!("Decimal, Number and Timestamp aren't support yet!")
            }
            "struct" => Struct,
            "list" => List,
            "sexp" => SExp,
            _ => SchemaDefined(value.to_case(Case::UpperCamel)),
        }
    }
}

impl From<String> for IonSchemaType {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}

impl From<&String> for IonSchemaType {
    fn from(value: &String) -> Self {
        value.as_str().into()
    }
}
