use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, Data, DeriveInput, Field, Fields, GenericParam, Generics, Type,
    TypeParamBound,
};

#[proc_macro_derive(BufSized)]
pub fn buf_sized(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = add_trait_bounds(input.generics, &parse_quote!(::buf_sized::BufSized));
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let buf_size = sum_buf_size(input.data);
    let expanded = quote! {
        impl #impl_generics ::buf_sized::BufSized for #name #ty_generics #where_clause {
            const BUF_SIZE: usize = #buf_size;
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn add_trait_bounds(mut generics: Generics, trait_name: &TypeParamBound) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(trait_name.clone());
        }
    }

    generics
}

fn sum_buf_size(data: Data) -> TokenStream {
    let mut result = quote! {0usize};

    match data {
        Data::Struct(structure) => match structure.fields {
            Fields::Named(fields) => {
                result.add_fields(fields.named);
            }
            Fields::Unit => (),
            Fields::Unnamed(fields) => {
                result.add_fields(fields.unnamed);
            }
        },
        Data::Enum(_) | Data::Union(_) => {
            unimplemented!("Enums and unions are not supported");
        }
    }

    result
}

trait TokenStreamExt {
    fn add_fields<T>(&mut self, fields: T)
    where
        T: IntoIterator<Item = Field>;
    fn add_field(&mut self, field: Field);
    fn add_type(&mut self, typ: Type);
}

impl TokenStreamExt for TokenStream {
    fn add_fields<T>(&mut self, fields: T)
    where
        T: IntoIterator<Item = Field>,
    {
        for field in fields {
            self.add_field(field);
        }
    }

    fn add_field(&mut self, field: Field) {
        self.add_type(field.ty);
    }

    fn add_type(&mut self, typ: Type) {
        self.extend(quote! { + <#typ as ::buf_sized::BufSized>::BUF_SIZE });
    }
}
