use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Field, Fields, Ident, Type};

#[proc_macro_derive(RunaComponent, attributes(runa, serialize_field, runa_editable, runa_runtime))]
pub fn derive_runa_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident.clone();
    let type_name = type_registration_name(&input);
    let constructor = type_factory_tokens(&input)
        .unwrap_or_else(|| quote! { ::std::default::Default::default() });
    let serialized_fields = serialized_fields_tokens(&input.data);
    let serialized_setters = serialized_setters_tokens(&input.data);

    TokenStream::from(quote! {
        impl ::runa_engine::runa_core::components::SerializedFieldAccess for #ident {
            fn serialized_fields(
                &self
            ) -> ::std::vec::Vec<::runa_engine::runa_core::components::SerializedField> {
                #serialized_fields
            }

            fn set_serialized_field(
                &mut self,
                field_name: &str,
                value: ::runa_engine::runa_core::components::SerializedFieldValue,
            ) -> bool {
                #serialized_setters
            }
        }

        impl ::runa_engine::runa_core::components::Component for #ident {
            fn as_any(&self) -> &dyn ::std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any {
                self
            }
        }

        impl ::runa_engine::RunaComponentType for #ident {
            fn runa_component_type_name() -> &'static str {
                #type_name
            }
        }

        impl #ident {
            pub fn register(engine: &mut ::runa_engine::Engine) -> ::runa_engine::TypeMetadata {
                engine.register_component_named_factory::<Self, _>(
                    <Self as ::runa_engine::RunaComponentType>::runa_component_type_name(),
                    || #constructor
                )
            }
        }

        impl ::runa_engine::RunaTypeRegistration for #ident {
            fn register(
                engine: &mut ::runa_engine::Engine
            ) -> ::runa_engine::TypeMetadata {
                #ident::register(engine)
            }
        }
    })
}

#[proc_macro_derive(RunaScript, attributes(runa, serialize_field, runa_editable, runa_runtime))]
pub fn derive_runa_script(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident.clone();
    let type_name = type_registration_name(&input);
    let constructor = type_factory_tokens(&input)
        .unwrap_or_else(|| quote! { ::std::default::Default::default() });
    let serialized_fields = serialized_fields_tokens(&input.data);
    let serialized_setters = serialized_setters_tokens(&input.data);

    TokenStream::from(quote! {
        impl ::runa_engine::runa_core::components::SerializedFieldAccess for #ident {
            fn serialized_fields(
                &self
            ) -> ::std::vec::Vec<::runa_engine::runa_core::components::SerializedField> {
                #serialized_fields
            }

            fn set_serialized_field(
                &mut self,
                field_name: &str,
                value: ::runa_engine::runa_core::components::SerializedFieldValue,
            ) -> bool {
                #serialized_setters
            }
        }

        impl ::runa_engine::RunaScriptType for #ident {
            fn runa_script_type_name() -> &'static str {
                #type_name
            }
        }

        impl #ident {
            pub fn register(engine: &mut ::runa_engine::Engine) -> ::runa_engine::TypeMetadata {
                engine.register_script_named_factory::<Self, _>(
                    <Self as ::runa_engine::RunaScriptType>::runa_script_type_name(),
                    || #constructor
                )
            }
        }

        impl ::runa_engine::RunaTypeRegistration for #ident {
            fn register(
                engine: &mut ::runa_engine::Engine
            ) -> ::runa_engine::TypeMetadata {
                #ident::register(engine)
            }
        }
    })
}

#[proc_macro_derive(RunaArchetype, attributes(runa))]
pub fn derive_runa_archetype(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident.clone();
    let archetype_name =
        archetype_name_override(&input).unwrap_or_else(|| to_snake_case(&ident.to_string()));

    TokenStream::from(quote! {
        impl ::runa_engine::RunaArchetype for #ident {
            fn key() -> ::runa_engine::ArchetypeKey {
                ::runa_engine::ArchetypeKey::new(#archetype_name)
            }

            fn create(
                world: &mut ::runa_engine::runa_core::ocs::World
            ) -> ::runa_engine::runa_core::ocs::ObjectId {
                #ident::create(world)
            }
        }

        impl #ident {
            pub fn archetype_key() -> ::runa_engine::ArchetypeKey {
                <Self as ::runa_engine::RunaArchetype>::key()
            }

            pub fn archetype_name() -> &'static str {
                #archetype_name
            }

            pub fn register(
                engine: &mut ::runa_engine::Engine
            ) -> ::runa_engine::ArchetypeMetadata {
                engine.register_archetype::<Self>()
            }
        }
    })
}

#[proc_macro_derive(RunaObjectDef, attributes(runa))]
pub fn derive_runa_object_def(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident.clone();
    let object_def_name =
        archetype_name_override(&input).unwrap_or_else(|| to_snake_case(&ident.to_string()));

    TokenStream::from(quote! {
        impl ::runa_engine::ObjectDefName for #ident {
            fn key() -> ::runa_engine::ObjectDefKey {
                ::runa_engine::ObjectDefKey::new(#object_def_name)
            }
        }

        impl #ident {
            pub fn object_def_key() -> ::runa_engine::ObjectDefKey {
                <Self as ::runa_engine::ObjectDefName>::key()
            }

            pub fn object_def_name() -> &'static str {
                #object_def_name
            }

            pub fn register(
                engine: &mut ::runa_engine::Engine
            ) -> ::runa_engine::ObjectDefMetadata {
                engine.register_object_def::<Self>()
            }
        }
    })
}

fn serialized_fields_tokens(data: &Data) -> TokenStream2 {
    let fields = serializable_fields(data);
    if fields.is_empty() {
        return quote! { ::std::vec::Vec::new() };
    }

    let items = fields.into_iter().filter_map(|field| {
        let ident = field.ident?;
        let name = ident.to_string();
        let access = quote! { self.#ident };
        serialized_field_getter(&field.ty, &name, access)
    });

    quote! { vec![ #(#items),* ] }
}

fn serialized_setters_tokens(data: &Data) -> TokenStream2 {
    let setters: Vec<_> = serializable_fields(data)
        .into_iter()
        .filter_map(|field| {
            let ident = field.ident?;
            let name = ident.to_string();
            serialized_field_setter(&field.ty, &name, quote! { self.#ident })
        })
        .collect();

    if setters.is_empty() {
        quote! { false }
    } else {
        quote! {
            match field_name {
                #(#setters)*
                _ => false,
            }
        }
    }
}

fn serializable_fields(data: &Data) -> Vec<Field> {
    let Data::Struct(DataStruct { fields, .. }) = data else {
        return Vec::new();
    };

    match fields {
        Fields::Named(named) => named
            .named
            .iter()
            .filter(|field| has_serialize_field_attr(field))
            .cloned()
            .collect(),
        _ => Vec::new(),
    }
}

fn has_serialize_field_attr(field: &Field) -> bool {
    field.attrs.iter().any(|attribute| {
        attribute.path().is_ident("serialize_field")
            || attribute.path().is_ident("runa_editable")
            || (attribute.path().is_ident("runa")
                && attribute
                    .parse_nested_meta(|meta| {
                        if meta.path.is_ident("serialize")
                            || meta.path.is_ident("field")
                            || meta.path.is_ident("editable")
                        {
                            Ok(())
                        } else {
                            Err(meta.error("unsupported runa field attribute"))
                        }
                    })
                    .is_ok())
    })
}

fn serialized_field_getter(ty: &Type, name: &str, access: TokenStream2) -> Option<TokenStream2> {
    let variant = serialized_value_variant(ty)?;
    let value = value_to_serialized_tokens(ty, access)?;
    Some(quote! {
        ::runa_engine::runa_core::components::SerializedField {
            name: #name.to_string(),
            value: ::runa_engine::runa_core::components::SerializedFieldValue::#variant(#value),
        }
    })
}

fn serialized_field_setter(ty: &Type, name: &str, access: TokenStream2) -> Option<TokenStream2> {
    let variant = serialized_value_variant(ty)?;
    let value = serialized_to_value_tokens(ty)?;
    Some(quote! {
        #name => {
            if let ::runa_engine::runa_core::components::SerializedFieldValue::#variant(value) = value {
                #access = #value;
                true
            } else {
                false
            }
        }
    })
}

fn serialized_value_variant(ty: &Type) -> Option<Ident> {
    let ident = type_ident_string(ty)?;
    let variant = match ident.as_str() {
        "bool" => "Bool",
        "i32" => "I32",
        "i64" => "I64",
        "u32" => "U32",
        "u64" => "U64",
        "f32" => "F32",
        "f64" => "F64",
        "String" => "String",
        "Vec2" => "Vec2",
        "Vec3" => "Vec3",
        _ => return None,
    };
    Some(Ident::new(variant, proc_macro2::Span::call_site()))
}

fn value_to_serialized_tokens(ty: &Type, access: TokenStream2) -> Option<TokenStream2> {
    let ident = type_ident_string(ty)?;
    match ident.as_str() {
        "bool" | "i32" | "i64" | "u32" | "u64" | "f32" | "f64" => Some(access),
        "String" => Some(quote! { #access.clone() }),
        "Vec2" => Some(quote! { [#access.x, #access.y] }),
        "Vec3" => Some(quote! { [#access.x, #access.y, #access.z] }),
        _ => None,
    }
}

fn serialized_to_value_tokens(ty: &Type) -> Option<TokenStream2> {
    let ident = type_ident_string(ty)?;
    match ident.as_str() {
        "bool" | "i32" | "i64" | "u32" | "u64" | "f32" | "f64" => Some(quote! { value }),
        "String" => Some(quote! { value }),
        "Vec2" => Some(quote! { ::runa_engine::runa_core::glam::Vec2::new(value[0], value[1]) }),
        "Vec3" => {
            Some(quote! { ::runa_engine::runa_core::glam::Vec3::new(value[0], value[1], value[2]) })
        }
        _ => None,
    }
}

fn type_ident_string(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string()),
        _ => None,
    }
}

fn archetype_name_override(input: &DeriveInput) -> Option<String> {
    for attribute in &input.attrs {
        if !attribute.path().is_ident("runa") {
            continue;
        }

        let mut value = None;
        let _ = attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                let literal: syn::LitStr = meta.value()?.parse()?;
                value = Some(literal.value());
            }
            Ok(())
        });

        if value.is_some() {
            return value;
        }
    }

    None
}

fn type_registration_name(input: &DeriveInput) -> String {
    archetype_name_override(input).unwrap_or_else(|| input.ident.to_string())
}

fn type_factory_tokens(input: &DeriveInput) -> Option<TokenStream2> {
    for attribute in &input.attrs {
        if !attribute.path().is_ident("runa") {
            continue;
        }

        let mut value = None;
        let _ = attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("factory") {
                let literal: syn::LitStr = meta.value()?.parse()?;
                value = Some(literal.parse::<TokenStream2>()?);
            }
            Ok(())
        });

        if value.is_some() {
            return value;
        }
    }

    None
}

fn to_snake_case(value: &str) -> String {
    let mut result = String::new();
    let mut previous_was_lowercase_or_digit = false;

    for ch in value.chars() {
        if ch.is_ascii_uppercase() {
            if previous_was_lowercase_or_digit && !result.ends_with('_') {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
            previous_was_lowercase_or_digit = false;
        } else if ch.is_ascii_alphanumeric() {
            result.push(ch.to_ascii_lowercase());
            previous_was_lowercase_or_digit = true;
        } else if !result.ends_with('_') && !result.is_empty() {
            result.push('_');
            previous_was_lowercase_or_digit = false;
        }
    }

    result.trim_matches('_').to_string()
}
