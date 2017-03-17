#![recursion_limit="1024"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

#[proc_macro_derive(EnumStr, attributes(wan))]
pub fn derive_language(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let ast = syn::parse_derive_input(&input.to_string()).unwrap();
  let gen = DeriveInput::new(ast).unwrap();
  gen.derive().parse().unwrap()
}

struct DeriveInput {
  ident: syn::Ident,
  variants: Vec<Variant>,
}

fn parse_variants(variants: Vec<syn::Variant>) -> Result<Vec<Variant>, String> {
  let mut ret = Vec::new();
  for variant in variants {
    if let Some(v) = Variant::new(variant)? {
      ret.push(v);
    }
  }
  Ok(ret)
}

impl DeriveInput {
  fn new(ast: syn::DeriveInput) -> Result<DeriveInput, String> {
    match ast.body {
      syn::Body::Enum(variants) => {
        Ok(DeriveInput {
          ident: ast.ident,
          variants: parse_variants(variants)?,
        })
      }
      _ => Err("#[derive(EnumStr)] is only supported for enum".into()),
    }
  }

  fn derive_display(&self) -> quote::Tokens {
    let ident = &self.ident;
    let body = self.variants.iter().map(|v| {
      let variant = &v.ident;
      let value = &v.value;
      quote!{
        #ident :: #variant => write!(f, #value)
      }
    });
    quote! {
      impl ::std::fmt::Display for #ident {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
          match *self {
            #( #body, )*
            #ident :: Unknown(ref s) => write!(f, "{}", s),
          }
        }
      }
    }
  }

  fn derive_from_str(&self) -> quote::Tokens {
    let ident = &self.ident;
    let body = self.variants.iter().map(|v| {
      let variant = &v.ident;
      let value = &v.value;
      quote!{
        #value => Ok(#ident :: #variant)
      }
    });
    quote!{
      impl ::std::str::FromStr for #ident {
        type Err = String;
        fn from_str(s: &str) -> ::std::result::Result<#ident, Self::Err> {
          match s {
            #( #body, )*
            s => Ok(#ident :: Unknown(s.to_owned())),
          }
        }
      }
    }
  }

  fn derive_serde(&self) -> quote::Tokens {
    let ident = &self.ident;
    let ident_str = format!("enum {}", ident.as_ref());

    let serialize = quote! {
      impl ::serde::Serialize for #ident {
        fn serialize<S>(&self, s: S) -> ::std::result::Result<S::Ok, S::Error>
          where S: ::serde::Serializer {
          s.serialize_str(&self.to_string())
        }
      }
    };

    let deserialize = quote! {
      impl ::serde::Deserialize for #ident {
        fn deserialize<D>(d: D) -> ::std::result::Result<#ident, D::Error>
          where D: ::serde::Deserializer {
          struct Visitor;
          impl ::serde::de::Visitor for Visitor {
            type Value = #ident;
            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
              formatter.write_str(#ident_str)
            }
            fn visit_str<E>(self, s: &str) -> ::std::result::Result<Self::Value, E>
              where E: ::serde::de::Error {
              ::std::str::FromStr::from_str(s).map_err(|e| E::custom(e))
            }
          }
          d.deserialize(Visitor)
        }
      }
    };

    quote!{
      #serialize
      #deserialize
    }
  }

  fn derive(&self) -> quote::Tokens {
    let display = self.derive_display();
    let from_str = self.derive_from_str();
    let serde = self.derive_serde();
    quote! {
      #display
      #from_str
      #serde
    }
  }
}

struct Variant {
  ident: syn::Ident,
  value: String,
}

impl Variant {
  fn new(variant: syn::Variant) -> Result<Option<Variant>, String> {
    let mut value = None;
    for attr in variant.attrs {
      let attr: syn::Attribute = attr;
      if attr.name() != "wan" {
        continue;
      }

      match attr.value {
        syn::MetaItem::List(ident, items) => {
          match ident.as_ref() {
            "wan" => {
              for item in items {
                match item {
                  syn::NestedMetaItem::MetaItem(syn::MetaItem::NameValue(ident,
                                                                         syn::Lit::Str(s, _))) => {
                    match ident.as_ref() {
                      "value" => value = Some(s.to_owned()),
                      ident => return Err(format!("'#[wan({})]' is invalid attribute item", ident)),
                    }
                  }
                  val => return Err(format!("invalid form in attribute: {:?}", val)),
                }
              }
            }
            ident => return Err(format!("'{}' is invalid attribute name", ident)),
          }
        }
        val => return Err(format!("invalid form in attribute: {:?}", val)),
      }
    }

    let ident = variant.ident;

    Ok(value.map(move |value| {
      Variant {
        ident: ident,
        value: value,
      }
    }))
  }
}
