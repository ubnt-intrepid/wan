#![recursion_limit="1024"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

#[proc_macro_derive(WanLanguageList, attributes(wan))]
pub fn derive_language_list(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let ast = syn::parse_derive_input(&input.to_string()).unwrap();
  let gen = DeriveInput::new(ast).unwrap().derive();
  gen.parse().unwrap()
}

struct DeriveInput {
  ident: syn::Ident,
  variants: Vec<Variant>,
}

impl DeriveInput {
  fn new(ast: syn::DeriveInput) -> Result<DeriveInput, String> {
    match ast.body {
      syn::Body::Enum(_variants) => {
        let mut variants = Vec::new();
        for variant in _variants {
          if let Some(v) = Variant::new(variant)? {
            variants.push(v);
          }
        }
        Ok(DeriveInput {
          ident: ast.ident,
          variants: variants,
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

  fn derive_get_default_compiler(&self) -> quote::Tokens {
    let ident = &self.ident;
    let body = self.variants.iter().map(|v| {
      let variant = &v.ident;
      let compiler = &v.compiler;
      quote!{
        #ident :: #variant => Some(#compiler)
      }
    });
    quote!{
      impl GetDefaultCompiler for #ident {
        fn get_default_compiler(&self) -> Option<&'static str> {
          match *self {
            #( #body, )*
            _ => None,
          }
        }
      }
    }
  }

  fn derive_from_extension(&self) -> quote::Tokens {
    let ident = &self.ident;
    let body = self.variants.iter().map(|v| {
      let variant = &v.ident;
      let extensions = &v.extensions;
      quote!{
        #(#extensions)|* => Ok(#ident :: #variant)
      }
    });
    quote!{
      impl FromExtension for Language {
        type Err = ::Error;
        fn from_extension(ext: &str) -> Result<Self> {
          match ext {
            #( #body, )*
            ext => Err(format!("Failed to guess filetype: '{}' is unknown extension", ext).into()),
          }
        }
      }
    }
  }

  fn derive(&self) -> quote::Tokens {
    let display = self.derive_display();
    let from_str = self.derive_from_str();
    let serde = self.derive_serde();
    let get_default_compiler = self.derive_get_default_compiler();
    let from_extension = self.derive_from_extension();
    quote! {
      #display
      #from_str
      #serde
      #from_extension
      #get_default_compiler
    }
  }
}

struct Variant {
  ident: syn::Ident,
  value: String,
  compiler: String,
  extensions: Vec<String>,
}

impl Variant {
  fn new(variant: syn::Variant) -> Result<Option<Variant>, String> {
    let mut value = None;
    let mut compiler = None;
    let mut ext = None;

    for attr in variant.attrs {
      let (ident, items) = match attr.value {
        syn::MetaItem::List(ident, items) => (ident, items),
        val => return Err(format!("invalid form in attribute: {:?}", val)),
      };

      if ident.as_ref() != "wan" {
        return Err(format!("'{}' is invalid attribute name", ident.as_ref()));
      }

      for item in items {
        match item {
          syn::NestedMetaItem::MetaItem(syn::MetaItem::Word(ident)) => {
            if ident.as_ref() == "ignore" {
              return Ok(None);
            }
          }
          syn::NestedMetaItem::MetaItem(syn::MetaItem::NameValue(ident, syn::Lit::Str(s, _))) => {
            match ident.as_ref() {
              "value" => value = Some(s.to_owned()),
              "compiler" => compiler = Some(s.to_owned()),
              "ext" => ext = Some(s.to_owned()),
              ident => return Err(format!("'#[wan({})]' is invalid attribute item", ident)),
            }
          }
          val => return Err(format!("invalid form in attribute: {:?}", val)),
        }
      }
    }

    let ident = variant.ident;
    let value = value.unwrap_or_else(|| ident.as_ref().to_owned());
    let compiler = compiler.unwrap_or_else(|| format!("{}-head", ident.as_ref().to_lowercase()));
    let ext = ext.unwrap_or_default();
    let extensions = ext.split(",").map(ToOwned::to_owned).collect();

    Ok(Some(Variant {
      ident: ident,
      value: value,
      compiler: compiler,
      extensions: extensions,
    }))
  }
}
