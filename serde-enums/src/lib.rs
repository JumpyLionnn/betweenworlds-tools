extern crate proc_macro;
use syn::{parse_macro_input, DeriveInput, Fields};
use quote::quote;

#[proc_macro_derive(SerdeEnum)]
pub fn derive_trait(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let name = ast.ident;
    let tag = "type";
    let mut current_enum_expression = 0u64;
    let mut arms = Vec::new();
    let mut enum_values = Vec::new();
    if let syn::Data::Enum(data) = ast.data {
        for variant in data.variants {
            let ident = variant.ident;
            let fields = desserialize_enum_fields(variant.fields);
            let streams: &[proc_macro2::TokenStream] = &[quote! { #ident }, fields];
            let arm = quote! {#current_enum_expression => Ok(<#name>::#(#streams)*)};
            arms.push(arm);
            let enum_value = current_enum_expression.to_string();
            enum_values.push(quote! {#enum_value});
            current_enum_expression += 1;
        }
        let expanded = quote! {
            impl<'de> Deserialize<'de> for #name {
                fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                    let value = Value::deserialize(d)?;
                    let enum_value = value.get(#tag).ok_or(serde::de::Error::missing_field(#tag))?;
                    let enum_value = Value::as_u64(enum_value).ok_or(serde::de::Error::custom(format!("Unexpected `{}`, expected uint", enum_value.to_string())))?;
                    match enum_value {
                        #(#arms),*,
                        _ => Err(serde::de::Error::unknown_variant(&enum_value.to_string(), &[#(#enum_values),*]))
                    }
                }
            }

            
        };
        proc_macro::TokenStream::from(expanded)
    }
    else {
        panic!("#[derive(SerdeEnum)] is only defined for enums!");
    }
    

}

fn desserialize_enum_fields(fields: Fields) -> proc_macro2::TokenStream {
    match fields {
        Fields::Named(_fields) => unimplemented!(),
        Fields::Unnamed(fields) => {
            let mut values = Vec::new();
            for field in fields.unnamed {
                let typ = field.ty;
                let field = quote!{
                    match <#typ>::deserialize(value) {
                        Ok(value) => value,
                        Err(error) => Err(serde::de::Error::custom(format!("{}", error)))?,
                    }};
                values.push(field);
            }
            quote! {(#(#values),*)}
        },
        Fields::Unit => quote! {}
    }
}

