use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Type};

fn get_opt_type(ty: &Type) -> Option<&Type> {
  if let Type::Path(syn::TypePath {
    qself: None,
    path: syn::Path {
      segments: punc_seg,
      leading_colon: _,
    },
  }) = ty
  {
    match punc_seg.first() {
      None => None,
      Some(path_segment) => {
        if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
          colon2_token: _,
          lt_token: _,
          args,
          gt_token: _,
        }) = &path_segment.arguments
        {
          match args.first() {
            Some(syn::GenericArgument::Type(t)) => {
              if path_segment.ident == "Option" {
                Some(t)
              } else {
                None
              }
            }
            _ => None,
          }
        } else {
          None
        }
      }
    }
  } else {
    None
  }
}

fn is_opt(ty: &Type) -> bool {
  get_opt_type(ty).is_some()
}

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
  let builder_types = field_types.iter().map(|f| {
    let typ = get_opt_type(f);
    match typ {
      None => quote!(::std::option::Option<#f>),
      Some(_) => quote!(#f),
    }
  });
  let field_method_type = field_types.iter().map(|f| {
    let typ = get_opt_type(f);
    match typ {
      None => f,
      Some(t) => t,
    }
  });
  let empty_checks = field_names.iter().zip(field_types.iter()).map(|(n, ty)| {
    if is_opt(ty) {
      quote!()
    } else {
      quote!(
          if let ::std::option::Option::None = self.#n {
            let err: ::std::boxed::Box<dyn ::std::error::Error> = ::std::string::String::from(
              format!("field {} is missing", stringify!(#n))).into();
            return ::std::result::Result::Err(err);
          }
      )
    }
  });
  let final_inst = field_names.iter().zip(field_types.iter()).map(|(n, ty)| {
    if is_opt(ty) {
      quote!(
        if self.#n.is_none() {
          ::std::option::Option::None
        } else {
          ::std::option::Option::Some(self.#n.take().unwrap())
        }
      )
    } else {
      quote!(self.#n.take().unwrap())
    }
  });

  quote!(
    pub struct #builder_name {
      #(#field_names: #builder_types,)*
    }

    impl #builder_name {
      #(
        fn #field_names(&mut self, val: #field_method_type) -> &mut Self {
          self.#field_names = ::std::option::Option::Some(val);
          self
        }

      )*

      pub fn build(&mut self) -> ::std::result::Result<#struct_name, ::std::boxed::Box<dyn ::std::error::Error>> {
        #(
          #empty_checks
        )*


        ::std::result::Result::Ok(#struct_name {
          #(#field_names: #final_inst,)*
        })
      }
    }


    impl #struct_name {
      pub fn builder() -> #builder_name {
        #builder_name {
          #(#field_names: ::std::option::Option::None),*
        }
      }
    }
  )
  .into()
}
