use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::{
    Generics, Ident, ItemStruct, Token, Type, WhereClause, WherePredicate,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

struct DecomposeArgs {
    new_name: Ident,
    mode: DecomposeMode,
    derives: Vec<Ident>,
    ref_derives: Vec<Ident>,
    gen_refs: bool,
}

enum DecomposeMode {
    Include(Vec<Ident>),
    Exclude(Vec<Ident>),
}

impl Parse for DecomposeArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let new_name: Ident = input.parse()?;
        let _comma: Token![,] = input.parse()?;

        let ident: Ident = input.parse()?;
        let mode_name = ident.to_string();

        let content;
        syn::parenthesized!(content in input);
        let fields: Punctuated<Ident, Token![,]> =
            content.parse_terminated(Ident::parse, Token![,])?;

        if fields.is_empty() {
            return Err(syn::Error::new(
                ident.span(),
                format!("`{}` list must not be empty", mode_name),
            ));
        }

        let mode = match mode_name.as_str() {
            "include" => DecomposeMode::Include(fields.into_iter().collect()),
            "exclude" => DecomposeMode::Exclude(fields.into_iter().collect()),
            _ => {
                return Err(syn::Error::new(
                    ident.span(),
                    "expected `include` or `exclude`",
                ));
            }
        };

        let mut derives = Vec::new();
        let mut ref_derives = Vec::new();
        let mut gen_refs = false;

        while input.peek(Token![,]) {
            let _comma: Token![,] = input.parse()?;
            let kw: Ident = input.parse()?;
            match kw.to_string().as_str() {
                "derive" => {
                    let derive_content;
                    syn::parenthesized!(derive_content in input);
                    derives = derive_content
                        .parse_terminated(Ident::parse, Token![,])?
                        .into_iter()
                        .collect();
                }
                "ref_derive" => {
                    let derive_content;
                    syn::parenthesized!(derive_content in input);
                    ref_derives = derive_content
                        .parse_terminated(Ident::parse, Token![,])?
                        .into_iter()
                        .collect();
                }
                "refs" => {
                    gen_refs = true;
                }
                other => {
                    return Err(syn::Error::new(
                        kw.span(),
                        format!(
                            "expected `derive`, `ref_derive`, or `refs`, got `{}`",
                            other
                        ),
                    ));
                }
            }
        }

        Ok(DecomposeArgs {
            new_name,
            mode,
            derives,
            ref_derives,
            gen_refs,
        })
    }
}

#[proc_macro_attribute]
pub fn decompose(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as DecomposeArgs);
    let input = parse_macro_input!(item as ItemStruct);

    let original_name = &input.ident;
    let new_name = &args.new_name;
    let excluded_name = Ident::new(&format!("{}Excluded", new_name), new_name.span());
    let vis = &input.vis;
    let generics = &input.generics;

    let derive_attr = if args.derives.is_empty() {
        quote! {}
    } else {
        let traits = &args.derives;
        quote! { #[derive(#(#traits),*)] }
    };

    let ref_derive_attr = if args.ref_derives.is_empty() {
        quote! {}
    } else {
        let traits = &args.ref_derives;
        quote! { #[derive(#(#traits),*)] }
    };

    let fields = match &input.fields {
        syn::Fields::Named(named) => &named.named,
        _ => {
            return syn::Error::new(
                input.ident.span(),
                "decompose only supports structs with named fields",
            )
            .to_compile_error()
            .into();
        }
    };

    let field_names: HashSet<String> = fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap().to_string())
        .collect();

    let (include_idents, exclude_idents) = match &args.mode {
        DecomposeMode::Include(incl) => {
            for name in incl {
                if !field_names.contains(&name.to_string()) {
                    return syn::Error::new(
                        name.span(),
                        format!("field `{}` not found on struct `{}`", name, original_name),
                    )
                    .to_compile_error()
                    .into();
                }
            }
            let incl_set: HashSet<String> = incl.iter().map(|i| i.to_string()).collect();
            let inc: Vec<&syn::Field> = fields
                .iter()
                .filter(|f| incl_set.contains(&f.ident.as_ref().unwrap().to_string()))
                .collect();
            let exc: Vec<&syn::Field> = fields
                .iter()
                .filter(|f| !incl_set.contains(&f.ident.as_ref().unwrap().to_string()))
                .collect();
            (inc, exc)
        }
        DecomposeMode::Exclude(excl) => {
            for name in excl {
                if !field_names.contains(&name.to_string()) {
                    return syn::Error::new(
                        name.span(),
                        format!("field `{}` not found on struct `{}`", name, original_name),
                    )
                    .to_compile_error()
                    .into();
                }
            }
            let excl_set: HashSet<String> = excl.iter().map(|i| i.to_string()).collect();
            let inc: Vec<&syn::Field> = fields
                .iter()
                .filter(|f| !excl_set.contains(&f.ident.as_ref().unwrap().to_string()))
                .collect();
            let exc: Vec<&syn::Field> = fields
                .iter()
                .filter(|f| excl_set.contains(&f.ident.as_ref().unwrap().to_string()))
                .collect();
            (inc, exc)
        }
    };

    if include_idents.is_empty() {
        return syn::Error::new(
            original_name.span(),
            "decompose would result in an empty new struct",
        )
        .to_compile_error()
        .into();
    }
    if exclude_idents.is_empty() {
        return syn::Error::new(
            original_name.span(),
            "decompose would result in an empty excluded struct",
        )
        .to_compile_error()
        .into();
    }

    let include_names: Vec<&Ident> = include_idents
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect();
    let exclude_names: Vec<&Ident> = exclude_idents
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect();

    let include_tys: Vec<&Type> = include_idents.iter().map(|f| &f.ty).collect();
    let exclude_tys: Vec<&Type> = exclude_idents.iter().map(|f| &f.ty).collect();

    let include_vis: Vec<&syn::Visibility> = include_idents.iter().map(|f| &f.vis).collect();
    let exclude_vis: Vec<&syn::Visibility> = exclude_idents.iter().map(|f| &f.vis).collect();

    let include_generics = filter_generics(generics, &include_tys);
    let exclude_generics = filter_generics(generics, &exclude_tys);

    let include_generics_params = &include_generics.params;
    let exclude_generics_params = &exclude_generics.params;
    let include_where = &include_generics.where_clause;
    let exclude_where = &exclude_generics.where_clause;

    let inlined_generics = &generics.params;

    let include_angled = if include_generics_params.is_empty() {
        quote! {}
    } else {
        quote! { <#include_generics_params> }
    };
    let exclude_angled = if exclude_generics_params.is_empty() {
        quote! {}
    } else {
        quote! { <#exclude_generics_params> }
    };
    let orig_angled = if inlined_generics.is_empty() {
        quote! {}
    } else {
        quote! { <#inlined_generics> }
    };
    let orig_impl = if inlined_generics.is_empty() {
        quote! { impl #original_name #orig_angled }
    } else {
        quote! { impl<#inlined_generics> #original_name #orig_angled }
    };

    let decompose_new_fields: Vec<_> = include_names
        .iter()
        .map(|name| {
            quote! { #name: self.#name }
        })
        .collect();
    let decompose_excluded_fields: Vec<_> = exclude_names
        .iter()
        .map(|name| {
            quote! { #name: self.#name }
        })
        .collect();
    let compose_inc_fields: Vec<_> = include_names
        .iter()
        .map(|name| {
            quote! { #name: inc.#name }
        })
        .collect();
    let compose_exc_fields: Vec<_> = exclude_names
        .iter()
        .map(|name| {
            quote! { #name: exc.#name }
        })
        .collect();

    let ref_output = if args.gen_refs {
        let new_name_ref = Ident::new(&format!("{}Ref", new_name), new_name.span());
        let excluded_name_ref = Ident::new(&format!("{}ExcludedRef", new_name), new_name.span());

        let include_ref_tys: Vec<_> = include_tys
            .iter()
            .map(|ty| {
                quote! { &'a #ty }
            })
            .collect();
        let exclude_ref_tys: Vec<_> = exclude_tys
            .iter()
            .map(|ty| {
                quote! { &'a #ty }
            })
            .collect();

        let ref_include_angled = if include_generics_params.is_empty() {
            quote! { <'a> }
        } else {
            quote! { <'a, #include_generics_params> }
        };
        let ref_exclude_angled = if exclude_generics_params.is_empty() {
            quote! { <'a> }
        } else {
            quote! { <'a, #exclude_generics_params> }
        };
        let ref_method_angled = if inlined_generics.is_empty() {
            quote! { <'a> }
        } else {
            quote! { <'a, #inlined_generics> }
        };

        let decompose_ref_new_fields: Vec<_> = include_names
            .iter()
            .map(|name| {
                quote! { #name: &self.#name }
            })
            .collect();
        let decompose_ref_excluded_fields: Vec<_> = exclude_names
            .iter()
            .map(|name| {
                quote! { #name: &self.#name }
            })
            .collect();

        quote! {
            #ref_derive_attr
            #[allow(dead_code)]
            #vis struct #new_name_ref #ref_include_angled #include_where {
                #(#include_vis #include_names: #include_ref_tys),*
            }

            #ref_derive_attr
            #[allow(dead_code)]
            #vis struct #excluded_name_ref #ref_exclude_angled #exclude_where {
                #(#exclude_vis #exclude_names: #exclude_ref_tys),*
            }

            impl #original_name #orig_angled {
                pub fn decompose_ref #ref_method_angled (&'a self) -> (#new_name_ref #ref_include_angled, #excluded_name_ref #ref_exclude_angled) {
                    (
                        #new_name_ref {
                            #(#decompose_ref_new_fields),*
                        },
                        #excluded_name_ref {
                            #(#decompose_ref_excluded_fields),*
                        },
                    )
                }
            }
        }
    } else {
        quote! {}
    };

    let output = quote! {
        #input

        #derive_attr
        #[allow(dead_code)]
        #vis struct #new_name #include_angled #include_where {
            #(#include_vis #include_names: #include_tys),*
        }

        #derive_attr
        #[allow(dead_code)]
        #vis struct #excluded_name #exclude_angled #exclude_where {
            #(#exclude_vis #exclude_names: #exclude_tys),*
        }

        #orig_impl {
            pub fn decompose(self) -> (#new_name #include_angled, #excluded_name #exclude_angled) {
                (
                    #new_name {
                        #(#decompose_new_fields),*
                    },
                    #excluded_name {
                        #(#decompose_excluded_fields),*
                    },
                )
            }

            pub fn compose(
                inc: #new_name #include_angled,
                exc: #excluded_name #exclude_angled,
            ) -> Self {
                Self {
                    #(#compose_inc_fields,)*
                    #(#compose_exc_fields,)*
                }
            }
        }

        #ref_output
    };

    output.into()
}

fn filter_generics<'a>(generics: &'a Generics, field_types: &[&'a Type]) -> Generics {
    let generic_idents: HashSet<String> = generics
        .params
        .iter()
        .filter_map(|p| match p {
            syn::GenericParam::Type(tp) => Some(tp.ident.to_string()),
            _ => None,
        })
        .collect();

    let used_idents: HashSet<String> = field_types
        .iter()
        .flat_map(|ty| collect_idents(ty))
        .filter(|ident| generic_idents.contains(ident))
        .collect();

    let mut new_generics = generics.clone();
    new_generics.params = new_generics
        .params
        .into_iter()
        .filter(|p| match p {
            syn::GenericParam::Type(tp) => used_idents.contains(&tp.ident.to_string()),
            _ => true,
        })
        .collect();

    if let Some(where_clause) = &mut new_generics.where_clause {
        filter_where_clause(where_clause, &used_idents);
    }

    new_generics
}

fn collect_idents(ty: &Type) -> Vec<String> {
    let mut result = Vec::new();
    collect_idents_inner(ty, &mut result);
    result
}

fn collect_idents_inner(ty: &Type, result: &mut Vec<String>) {
    match ty {
        Type::Path(type_path) => {
            if let Some(seg) = type_path.path.segments.last() {
                let name = seg.ident.to_string();
                if name.chars().next().map_or(false, |c| c.is_uppercase()) {
                    result.push(name);
                }
                if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                    for arg in &args.args {
                        if let syn::GenericArgument::Type(inner_ty) = arg {
                            collect_idents_inner(inner_ty, result);
                        }
                    }
                }
            }
        }
        Type::Reference(type_ref) => {
            collect_idents_inner(&type_ref.elem, result);
        }
        Type::Tuple(tuple) => {
            for elem in &tuple.elems {
                collect_idents_inner(elem, result);
            }
        }
        Type::Slice(slice) => {
            collect_idents_inner(&slice.elem, result);
        }
        _ => {}
    }
}

fn filter_where_clause(where_clause: &mut WhereClause, used_idents: &HashSet<String>) {
    where_clause.predicates = where_clause
        .predicates
        .clone()
        .into_iter()
        .filter(|pred| match pred {
            WherePredicate::Type(pred) => {
                let idents = collect_idents(&pred.bounded_ty);
                idents.iter().any(|i| used_idents.contains(i))
            }
            _ => true,
        })
        .collect();
}
