use anyhow::Context;
use duit_core::spec::{Spec, Widget};
use proc_macro2::Span;
use quote::quote;
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
};
use syn::Ident;

#[derive(Debug, argh::FromArgs)]
/// generate Rust code from Duit widget spec files
pub struct Args {
    #[argh(positional)]
    /// input YAML file
    input: PathBuf,
    #[argh(option, short = 'o')]
    /// output Rust file
    output: PathBuf,
    #[argh(switch)]
    /// whether to append to instead of overwrite the output file
    append: bool,
}

fn main() -> anyhow::Result<()> {
    let args: Args = argh::from_env();

    let spec = Spec::deserialize_from_str(&fs::read_to_string(&args.input)?)?;
    let code = generate_code(&spec);
    let code = rustfmt(&code)?;

    let mut output_file = OpenOptions::new()
        .append(args.append)
        .write(true)
        .create(true)
        .open(args.output)?;
    output_file.write_all(code.as_bytes())?;

    Ok(())
}

#[derive(Debug)]
struct Entry {
    widget_type: String,
    id: String,
}

fn generate_code(spec: &Spec) -> String {
    let mut entries = Vec::new();
    gather_entries(&spec.child, &mut entries);

    let ident = Ident::new(&spec.name, Span::call_site());
    let name = &spec.name;

    let mut struct_fields = Vec::new();
    let mut bindings = Vec::new();
    let mut match_arms = Vec::new();
    let mut inits = Vec::new();
    for entry in &entries {
        let id_str = &entry.id;
        let id = Ident::new(&entry.id, Span::call_site());
        let typ = Ident::new(&entry.widget_type, Span::call_site());
        struct_fields.push(quote! {
           pub #id: WidgetHandle<#typ>
        });
        bindings.push(quote! {
            let mut #id = None;
        });
        match_arms.push(quote! {
            #id_str => #id = Some(widget)
        });
        inits.push(quote! {
            #id: WidgetHandle::new(#id.unwrap_or_else(|| panic!("missing widget with ID '{}' (generated code not up to date)", #id_str)))
        })
    }

    let tokens = quote! {
        use ::duit::widgets::*;
        use ::duit::*;

        pub struct #ident {
            #(#struct_fields,)*
        }

        impl ::duit::InstanceHandle for #ident {
            fn name() -> &'static str {
                #name
            }

            fn init(widget_handles: Vec<(String, WidgetPodHandle)>) -> Self {
                #(#bindings)*
                for (name, widget) in widget_handles {
                    match name.as_str() {
                        #(#match_arms,)*
                        _ => {},
                    }
                }
                Self {
                    #(#inits,)*
                }
            }
        }
    };
    tokens.to_string()
}

fn gather_entries(widget: &Widget, entries: &mut Vec<Entry>) {
    if let Some(id) = widget.base_spec().and_then(|s| s.id.clone()) {
        entries.push(Entry {
            widget_type: widget.type_name().to_owned(),
            id,
        });
    }

    // Recurse
    for child in widget.children() {
        gather_entries(child, entries);
    }
}

fn rustfmt(code: &str) -> anyhow::Result<String> {
    let child = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("rustfmt not in PATH")?;

    child.stdin.unwrap().write_all(code.as_bytes())?;

    let mut buf = String::new();
    child.stdout.unwrap().read_to_string(&mut buf)?;
    Ok(buf)
}
