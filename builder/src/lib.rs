use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
  let ast: DeriveInput = syn::parse(input).unwrap();

  // verify that struct was derived

  let struct_data = if let Data::Struct(d) = ast.data {
    d
  } else {
    unimplemented!()
  };

  let struct_name = ast.ident;
  let builder_name = format_ident!("{}Builder", struct_name);

  let field_names: Vec<_> = struct_data
    .fields
    .iter()
    .map(|f| f.ident.clone().expect("does not work with unnamed field"))
    .collect();

  let field_types: Vec<_> = struct_data.fields.iter().map(|f| f.ty.clone()).collect();

  quote!(
    use ::std::error::Error;
    pub struct #builder_name {
      #(#field_names: Option<#field_types>,)*
    }

    impl #builder_name {
      #(
        fn #field_names(&mut self, #field_names: #field_types) -> &mut Self {
          self.#field_names = Some(#field_names);
          self
        }

      )*

      pub fn build(&mut self) -> Result<#struct_name, Box<dyn Error>> {
        #(
          if let None = self.#field_names {
            let err: Box<dyn Error> = String::from(format!("field {} is missing", stringify!(#field_names))).into();
            return Err(err);
          }
        )*

        Ok(#struct_name {
          #(#field_names: self.#field_names.take().unwrap(),)*
        })
      }
    }


    impl #struct_name {
      pub fn builder() -> #builder_name {
        #builder_name {
          #(#field_names: None),*
        }
      }
    }
  )
  .into()
}
