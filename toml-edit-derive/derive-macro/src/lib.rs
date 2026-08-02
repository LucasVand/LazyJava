use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    AngleBracketedGenericArguments, Attribute, Data, DeriveInput, Fields, GenericArgument, Ident,
    LitStr, PathArguments, Type, TypePath, Variant, Visibility, parse_macro_input,
};

#[proc_macro_derive(TomlEdit, attributes(toml_edit, serde))]
pub fn derive_toml_edit(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

fn expand(input: DeriveInput) -> syn::Result<TokenStream2> {
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "TomlEdit: generic types are not supported",
        ));
    }
    match &input.data {
        Data::Struct(_) => expand_struct(input),
        Data::Enum(e) => {
            let variants = e.variants.iter().cloned().collect::<Vec<_>>();
            expand_enum(input, variants)
        }
        _ => Err(syn::Error::new_spanned(
            &input.ident,
            "TomlEdit: only structs and enums supported",
        )),
    }
}

// -----------------------------------------------------------------------
// #[toml_edit(inline)] attribute — struct level
// -----------------------------------------------------------------------
#[derive(Default)]
struct TomlEditStructAttrs {
    inline: bool,
}

impl TomlEditStructAttrs {
    fn parse_attrs(attrs: &[Attribute]) -> Self {
        let mut out = TomlEditStructAttrs::default();
        for attr in attrs {
            if attr.path().is_ident("toml_edit") {
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("inline") {
                        out.inline = true;
                    }
                    Ok(())
                });
            }
        }
        out
    }
}

// -----------------------------------------------------------------------
// Struct expansion (the main case)
// -----------------------------------------------------------------------
fn expand_struct(input: DeriveInput) -> syn::Result<TokenStream2> {
    let vis = &input.vis;
    let struct_name = &input.ident;
    let edit_name = format_ident!("{}TomlEdit", struct_name);
    let ref_name = format_ident!("{}TomlEditView", struct_name);
    let mut_name = format_ident!("{}TomlEditMut", struct_name);

    let struct_attrs = SerdeStructAttrs::parse_attrs(&input.attrs);
    let tomledit_attrs = TomlEditStructAttrs::parse_attrs(&input.attrs);

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => f.named.iter().cloned().collect::<Vec<_>>(),
            _ => {
                return Err(syn::Error::new_spanned(
                    &struct_name,
                    "TomlEdit: only named fields supported",
                ));
            }
        },
        _ => unreachable!(),
    };

    let mut edit_methods = Vec::new();
    let mut proxy_reads = Vec::new();
    let mut proxy_writes = Vec::new();
    let mut into_item_stmts = Vec::new();

    let doc = quote!(self.doc);
    let doc_tbl = quote!(&mut *self.doc);
    let tbl = quote!(self.table);

    for field in &fields {
        let fname = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        let field_attrs = SerdeFieldAttrs::parse_attrs(&field.attrs);
        if field_attrs.skip {
            continue;
        }
        let fname_mut = format_ident!("{}_mut", fname);

        if field_attrs.flatten {
            if !field_attrs.skip_deserializing {
                edit_methods.push(flatten_read(
                    &quote!(self.doc.as_item().as_table_like()),
                    fname,
                    ty,
                ));
                proxy_reads.push(flatten_read(&quote!(Some(&*self.table)), fname, ty));
            }
            if !field_attrs.skip_serializing {
                edit_methods.push(flatten_mut(
                    &quote!(self.doc.as_item_mut().as_table_like_mut()),
                    &fname_mut,
                    ty,
                ));
                proxy_writes.push(flatten_mut(&quote!(Some(&mut *self.table)), &fname_mut, ty));
                into_item_stmts.push(flatten_into(&quote!(self.#fname), ty));
            }
            continue;
        }

        let fkey = resolve_key(
            &fname.to_string(),
            &field_attrs,
            struct_attrs.rename_all.as_deref(),
        );

        if !field_attrs.skip_deserializing {
            edit_methods.push(field_read(&doc, &fkey, fname, ty));
            proxy_reads.push(field_read(&tbl, &fkey, fname, ty));
        }
        if !field_attrs.skip_serializing {
            edit_methods.push(field_write_or_entry(&doc_tbl, &fkey, &fname_mut, ty));
            proxy_writes.push(field_write_or_entry(&tbl, &fkey, &fname_mut, ty));
            into_item_stmts.push(gen_into_stmt(&fkey, &quote!(self.#fname), ty));
        }
    }

    let proxy_structs =
        gen_view_mut_structs(vis, &ref_name, &mut_name, &proxy_reads, &proxy_writes);

    let into_item_body = if tomledit_attrs.inline {
        quote! {
            let mut __t = ::toml_edit_derive::Table::new();
            #(#into_item_stmts)*
            ::toml_edit_derive::inline_item(__t.into_iter().map(|(__k, __v)| (::std::string::ToString::to_string(&__k), __v)))
        }
    } else {
        quote! {
            let mut __t = ::toml_edit_derive::Table::new();
            #(#into_item_stmts)*
            ::toml_edit_derive::Item::Table(__t)
        }
    };

    let empty_item_fn = if tomledit_attrs.inline {
        quote! {
            fn empty_item() -> ::toml_edit_derive::Item {
                ::toml_edit_derive::inline_item(::std::vec::Vec::<(String, ::toml_edit_derive::Item)>::new())
            }
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        #[derive(Debug, Clone)]
        #vis struct #edit_name {
            pub doc: ::toml_edit_derive::DocumentMut,
        }

        impl #edit_name {
            pub fn parse(s: &str) -> Result<Self, ::toml_edit_derive::TomlError> {
                let doc = s.parse::<::toml_edit_derive::DocumentMut>()?;
                Ok(Self { doc })
            }

            pub fn to_toml_string(&self) -> String {
                self.doc.to_string()
            }

            #(#edit_methods)*
        }

        impl std::str::FromStr for #edit_name {
            type Err = ::toml_edit_derive::TomlError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::parse(s)
            }
        }

        #proxy_structs

        impl ::toml_edit_derive::TomlEditable for #struct_name {
            type View<'a> = #ref_name<'a>;
            type Mut<'a> = #mut_name<'a>;

            fn from_table_like(table: &dyn ::toml_edit_derive::TableLike) -> Option<#ref_name<'_>> {
                Some(#ref_name { table })
            }
            fn from_table_like_mut(table: &mut dyn ::toml_edit_derive::TableLike) -> Option<#mut_name<'_>> {
                Some(#mut_name { table })
            }
            fn into_item(self) -> ::toml_edit_derive::Item {
                #into_item_body
            }
            #empty_item_fn
        }
    })
}

// -----------------------------------------------------------------------
// Enum expansion (unchanged from original)
// -----------------------------------------------------------------------
fn expand_enum(input: DeriveInput, variants: Vec<Variant>) -> syn::Result<TokenStream2> {
    let all_unit = variants.iter().all(|v| matches!(v.fields, Fields::Unit));
    let struct_attrs = SerdeStructAttrs::parse_attrs(&input.attrs);
    if all_unit {
        expand_unit_enum(&input.ident, &variants, &struct_attrs)
    } else {
        expand_struct_enum(&input, &variants, &struct_attrs)
    }
}

fn expand_unit_enum(
    enum_name: &Ident,
    variants: &[Variant],
    attrs: &SerdeStructAttrs,
) -> syn::Result<TokenStream2> {
    let mut from_arms = Vec::new();
    let mut to_arms = Vec::new();
    for variant in variants {
        let vname = &variant.ident;
        let vattrs = SerdeFieldAttrs::parse_attrs(&variant.attrs);
        if vattrs.skip {
            continue;
        }
        let key = resolve_variant_key(&vname.to_string(), &vattrs, attrs.rename_all.as_deref());
        from_arms.push(quote! { #key => Some(#enum_name::#vname), });
        to_arms.push(quote! { #enum_name::#vname => #key, });
    }
    Ok(quote! {
        impl ::toml_edit_derive::TomlEditValue for #enum_name {
            fn from_value(v: &::toml_edit_derive::Value) -> Option<#enum_name> {
                v.as_str().and_then(|s| match s { #(#from_arms)* _ => None, })
            }
            fn into_value(self) -> ::toml_edit_derive::Value {
                ::toml_edit_derive::Value::from(match self { #(#to_arms)* })
            }
        }
    })
}

fn expand_struct_enum(
    input: &DeriveInput,
    variants: &[Variant],
    attrs: &SerdeStructAttrs,
) -> syn::Result<TokenStream2> {
    let vis = &input.vis;
    let struct_name = &input.ident;
    let edit_name = format_ident!("{}TomlEdit", struct_name);
    let ref_name = format_ident!("{}TomlEditView", struct_name);
    let mut_name = format_ident!("{}TomlEditMut", struct_name);

    let tag_key = attrs.tag.as_deref().ok_or_else(|| {
        syn::Error::new_spanned(
            struct_name,
            "TomlEdit: enum with struct variants requires #[serde(tag = \"...\")]",
        )
    })?;

    let mut proxy_defs = Vec::new();
    let mut ref_variants = Vec::new();
    let mut ref_from_arms = Vec::new();
    let mut mut_methods = Vec::new();
    let mut into_arms = Vec::new();
    let mut edit_methods = Vec::new();
    let mut tag_keys = std::collections::HashSet::new();

    for variant in variants {
        if matches!(variant.fields, Fields::Unnamed(_)) {
            return Err(syn::Error::new_spanned(
                &variant.ident,
                "TomlEdit: tuple enum variants not supported",
            ));
        }
        let vname = &variant.ident;
        let vattrs = SerdeFieldAttrs::parse_attrs(&variant.attrs);
        if vattrs.skip {
            continue;
        }
        let vkey = resolve_variant_key(&vname.to_string(), &vattrs, attrs.rename_all.as_deref());
        if !tag_keys.insert(vkey.clone()) {
            return Err(syn::Error::new_spanned(
                vname,
                format!(
                    "TomlEdit: enum variant tag '{}' collides with another variant",
                    vkey
                ),
            ));
        }
        let is_unit = matches!(variant.fields, Fields::Unit);
        let var_ref = format_ident!("{}{}TomlEditView", struct_name, vname);
        let var_mut = format_ident!("{}{}TomlEditMut", struct_name, vname);
        let method_name = pascal_to_snake(&vname.to_string());
        let method_ident = format_ident!("{}", method_name);
        let as_fn_mut = format_ident!("as_{}_mut", method_name);

        let mut proxy_reads = Vec::new();
        let mut proxy_writes = Vec::new();
        let mut field_names = Vec::new();
        let mut into_stmts = Vec::new();
        let tbl = quote!(self.table);

        if let Fields::Named(named) = &variant.fields {
            for field in &named.named {
                let fname = field.ident.as_ref().unwrap();
                let ty = &field.ty;
                let fattrs = SerdeFieldAttrs::parse_attrs(&field.attrs);
                if fattrs.skip {
                    continue;
                }
                let fkey = resolve_key(&fname.to_string(), &fattrs, attrs.rename_all.as_deref());
                if fkey == tag_key {
                    return Err(syn::Error::new_spanned(
                        fname,
                        format!(
                            "TomlEdit: field key '{}' collides with the enum tag key '{}'",
                            fkey, tag_key
                        ),
                    ));
                }
                let fname_mut = format_ident!("{}_mut", fname);
                if !fattrs.skip_deserializing {
                    proxy_reads.push(field_read(&tbl, &fkey, fname, ty));
                }
                if !fattrs.skip_serializing {
                    proxy_writes.push(field_write_or_entry(&tbl, &fkey, &fname_mut, ty));
                    into_stmts.push(gen_into_stmt(&fkey, &quote!(#fname), ty));
                }
                field_names.push(fname.clone());
            }
        }

        if is_unit {
            ref_variants.push(quote! { #vname, });
            ref_from_arms.push(quote! { Some(#vkey) => #ref_name::#vname, });
            into_arms.push(quote! {
                #struct_name::#vname => {
                    __t.insert(#tag_key, ::toml_edit_derive::Item::Value(::toml_edit_derive::Value::from(#vkey)));
                }
            });
            let tag_set = set_kv(
                &quote!(self.table),
                tag_key,
                &quote!(::toml_edit_derive::Item::Value(::toml_edit_derive::Value::from(#vkey))),
            );
            mut_methods.push(quote! { pub fn #method_ident(self) { #tag_set } });
            let is_fn = format_ident!("is_{}", method_name);
            edit_methods.push(quote! {
                pub fn #is_fn(&self) -> bool {
                    self.doc.get(#tag_key)
                        .and_then(|i| i.as_value())
                        .and_then(|v| v.as_str()) == Some(#vkey)
                }
            });
        } else {
            proxy_defs.push(gen_view_mut_structs(
                vis,
                &var_ref,
                &var_mut,
                &proxy_reads,
                &proxy_writes,
            ));
            ref_variants.push(quote! { #vname(#var_ref<'a>), });
            ref_from_arms.push(quote! { Some(#vkey) => #ref_name::#vname(#var_ref { table }), });
            into_arms.push(quote! {
                #struct_name::#vname { #(#field_names,)* } => {
                    __t.insert(#tag_key, ::toml_edit_derive::Item::Value(::toml_edit_derive::Value::from(#vkey)));
                    #(#into_stmts)*
                }
            });
            let tag_set = set_kv(
                &quote!(self.table),
                tag_key,
                &quote!(::toml_edit_derive::Item::Value(::toml_edit_derive::Value::from(#vkey))),
            );
            mut_methods.push(quote! {
                pub fn #method_ident(self) -> #var_mut<'a> {
                    #tag_set
                    #var_mut { table: self.table }
                }
                pub fn #as_fn_mut(self) -> Option<#var_mut<'a>> {
                    (self.table.get(#tag_key)
                        .and_then(|i| i.as_value())
                        .and_then(|v| v.as_str()) == Some(#vkey))
                    .then(|| #var_mut { table: self.table })
                }
            });
            let as_fn = format_ident!("as_{}", method_name);
            edit_methods.push(quote! {
                pub fn #as_fn(&self) -> Option<#var_ref<'_>> {
                    let table = self.doc.as_item().as_table_like()?;
                    (table.get(#tag_key)
                        .and_then(|i| i.as_value())
                        .and_then(|v| v.as_str()) == Some(#vkey))
                    .then(|| #var_ref { table })
                }
                pub fn #as_fn_mut(&mut self) -> Option<#var_mut<'_>> {
                    let table = self.doc.as_item_mut().as_table_like_mut()?;
                    (table.get(#tag_key)
                        .and_then(|i| i.as_value())
                        .and_then(|v| v.as_str()) == Some(#vkey))
                    .then(|| #var_mut { table })
                }
            });
        }
    }

    Ok(quote! {
        #(#proxy_defs)*

        #[derive(Debug, Clone, Copy)]
        #vis enum #ref_name<'a> {
            #(#ref_variants)*
            Unknown,
        }

        #vis struct #mut_name<'a> {
            pub table: &'a mut dyn ::toml_edit_derive::TableLike,
        }

        impl<'a> #mut_name<'a> {
            #(#mut_methods)*
        }

        impl ::core::fmt::Debug for #mut_name<'_> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct(::core::stringify!(#mut_name)).finish_non_exhaustive()
            }
        }

        #[derive(Debug, Clone)]
        #vis struct #edit_name {
            pub doc: ::toml_edit_derive::DocumentMut,
        }

        impl #edit_name {
            pub fn parse(s: &str) -> Result<Self, ::toml_edit_derive::TomlError> {
                let doc = s.parse::<::toml_edit_derive::DocumentMut>()?;
                Ok(Self { doc })
            }
            pub fn to_toml_string(&self) -> String { self.doc.to_string() }
            pub fn view(&self) -> #ref_name<'_> {
                <#struct_name as ::toml_edit_derive::TomlEditable>::from_item(self.doc.as_item())
                    .unwrap_or(#ref_name::Unknown)
            }
            pub fn as_mut_ref(&mut self) -> Option<#mut_name<'_>> {
                self.doc.as_item_mut().as_table_like_mut().map(|table| #mut_name { table })
            }
            #(#edit_methods)*
        }

        impl std::str::FromStr for #edit_name {
            type Err = ::toml_edit_derive::TomlError;
            fn from_str(s: &str) -> Result<Self, Self::Err> { Self::parse(s) }
        }

        impl ::toml_edit_derive::TomlEditable for #struct_name {
            type View<'a> = #ref_name<'a>;
            type Mut<'a> = #mut_name<'a>;
            fn from_table_like(table: &dyn ::toml_edit_derive::TableLike) -> Option<#ref_name<'_>> {
                Some(match table.get(#tag_key).and_then(|i| i.as_value()).and_then(|v| v.as_str()) {
                    #(#ref_from_arms)*
                    _ => #ref_name::Unknown,
                })
            }
            fn from_table_like_mut(table: &mut dyn ::toml_edit_derive::TableLike) -> Option<#mut_name<'_>> {
                Some(#mut_name { table })
            }
            fn into_item(self) -> ::toml_edit_derive::Item {
                ::toml_edit_derive::Item::Table({
                    let mut __t = ::toml_edit_derive::Table::new();
                    match self { #(#into_arms)* }
                    __t
                })
            }
        }
    })
}

// -----------------------------------------------------------------------
// Helpers — struct generation
// -----------------------------------------------------------------------
fn gen_view_struct(vis: &Visibility, name: &Ident, reads: &[TokenStream2]) -> TokenStream2 {
    quote! {
        #[derive(Clone, Copy)]
        #vis struct #name<'a> {
            pub table: &'a dyn ::toml_edit_derive::TableLike,
        }
        impl<'a> #name<'a> { #(#reads)* }
        impl ::core::fmt::Debug for #name<'_> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct(::core::stringify!(#name)).finish_non_exhaustive()
            }
        }
    }
}

fn gen_mut_struct(
    vis: &Visibility,
    name: &Ident,
    reads: &[TokenStream2],
    writes: &[TokenStream2],
) -> TokenStream2 {
    quote! {
        #vis struct #name<'a> {
            pub table: &'a mut dyn ::toml_edit_derive::TableLike,
        }
        impl<'a> #name<'a> { #(#reads)* #(#writes)* }
        impl ::core::fmt::Debug for #name<'_> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct(::core::stringify!(#name)).finish_non_exhaustive()
            }
        }
    }
}

fn gen_view_mut_structs(
    vis: &Visibility,
    ref_name: &Ident,
    mut_name: &Ident,
    reads: &[TokenStream2],
    writes: &[TokenStream2],
) -> TokenStream2 {
    let view = gen_view_struct(vis, ref_name, reads);
    let mut_struct = gen_mut_struct(vis, mut_name, reads, writes);
    quote! { #view #mut_struct }
}

fn unwrap_option(ty: &Type) -> &Type {
    single_generic_arg(ty, "Option").unwrap_or(ty)
}

fn field_read(src: &TokenStream2, fkey: &str, fname: &Ident, ty: &Type) -> TokenStream2 {
    let inner = unwrap_option(ty);
    quote! {
        pub fn #fname(&self) -> Option<<#inner as ::toml_edit_derive::TomlEditable>::View<'_>> {
            #src.get(#fkey).and_then(|item| <#inner as ::toml_edit_derive::TomlEditable>::from_item(item))
        }
    }
}

fn field_write_or_entry(
    entry_src: &TokenStream2,
    fkey: &str,
    fname_mut: &Ident,
    ty: &Type,
) -> TokenStream2 {
    let inner = unwrap_option(ty);
    quote! {
        pub fn #fname_mut(&mut self) -> ::toml_edit_derive::FieldEntry<'_, #inner> {
            ::toml_edit_derive::FieldEntry::new(#entry_src, #fkey)
        }
    }
}

fn set_kv(target: &TokenStream2, fkey: &str, item: &TokenStream2) -> TokenStream2 {
    quote! {
        {
            let __item = #item;
            match #target.get_mut(#fkey) {
                ::core::option::Option::Some(__slot) => { *__slot = __item; }
                ::core::option::Option::None => { let _ = #target.insert(#fkey, __item); }
            }
        }
    }
}

fn gen_into_stmt(fkey: &str, val: &TokenStream2, ty: &Type) -> TokenStream2 {
    if let Some(inner) = single_generic_arg(ty, "Option") {
        quote! {
            if let Some(__v) = #val {
                __t.insert(#fkey, <#inner as ::toml_edit_derive::TomlEditable>::into_item(__v));
            }
        }
    } else {
        quote! { __t.insert(#fkey, <#ty as ::toml_edit_derive::TomlEditable>::into_item(#val)); }
    }
}

fn flatten_read(tl_opt: &TokenStream2, fname: &Ident, ty: &Type) -> TokenStream2 {
    let field_name = fname.to_string();
    let err = format!(
        "TomlEdit: #[serde(flatten)] on `{field_name}` requires a struct type that derives TomlEdit, not a value type"
    );
    quote! {
        pub fn #fname(&self) -> Option<<#ty as ::toml_edit_derive::TomlEditable>::View<'_>> {
            const _: () = ::core::assert!(
                !<#ty as ::toml_edit_derive::TomlEditable>::IS_VALUE_TYPE,
                #err,
            );
            #tl_opt.and_then(|__t| <#ty as ::toml_edit_derive::TomlEditable>::from_table_like(__t))
        }
    }
}

fn flatten_mut(tl_opt: &TokenStream2, fname_mut: &Ident, ty: &Type) -> TokenStream2 {
    quote! {
        pub fn #fname_mut(&mut self) -> Option<<#ty as ::toml_edit_derive::TomlEditable>::Mut<'_>> {
            #tl_opt.and_then(|__t| <#ty as ::toml_edit_derive::TomlEditable>::from_table_like_mut(__t))
        }
    }
}

fn flatten_into(val: &TokenStream2, ty: &Type) -> TokenStream2 {
    let err = "TomlEdit: #[serde(flatten)] on a value type is not supported; \
               the field must be a struct with #[derive(TomlEdit)]";
    quote! {
        const _: () = ::core::assert!(
            !<#ty as ::toml_edit_derive::TomlEditable>::IS_VALUE_TYPE,
            #err,
        );
        if let ::toml_edit_derive::Item::Table(__ft) =
            <#ty as ::toml_edit_derive::TomlEditable>::into_item(#val)
        {
            for (__k, __v) in __ft {
                __t.insert(&__k, __v);
            }
        }
    }
}

// -----------------------------------------------------------------------
// Serde attribute parsing
// -----------------------------------------------------------------------
#[derive(Default)]
struct SerdeFieldAttrs {
    rename: Option<String>,
    skip: bool,
    skip_serializing: bool,
    skip_deserializing: bool,
    flatten: bool,
}

impl SerdeFieldAttrs {
    fn parse_attrs(attrs: &[Attribute]) -> Self {
        let mut out = SerdeFieldAttrs::default();
        for attr in attrs {
            if attr.path().is_ident("serde") {
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("rename") {
                        let v: LitStr = meta.value()?.parse()?;
                        out.rename = Some(v.value());
                    } else if meta.path.is_ident("skip") {
                        out.skip = true;
                    } else if meta.path.is_ident("skip_serializing") {
                        out.skip_serializing = true;
                    } else if meta.path.is_ident("skip_deserializing") {
                        out.skip_deserializing = true;
                    } else if meta.path.is_ident("flatten") {
                        out.flatten = true;
                    }
                    Ok(())
                });
            }
        }
        out
    }
}

#[derive(Default)]
struct SerdeStructAttrs {
    rename_all: Option<String>,
    tag: Option<String>,
}

impl SerdeStructAttrs {
    fn parse_attrs(attrs: &[Attribute]) -> Self {
        let mut out = SerdeStructAttrs::default();
        for attr in attrs {
            if !attr.path().is_ident("serde") {
                continue;
            }
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename_all") {
                    let v: LitStr = meta.value()?.parse()?;
                    out.rename_all = Some(v.value());
                } else if meta.path.is_ident("tag") {
                    let v: LitStr = meta.value()?.parse()?;
                    out.tag = Some(v.value());
                }
                Ok(())
            });
        }
        out
    }
}

// -----------------------------------------------------------------------
// Name resolution
// -----------------------------------------------------------------------
fn apply_rename_all(name: &str, rule: &str) -> String {
    match rule {
        "lowercase" => name.to_lowercase(),
        "UPPERCASE" => name.to_uppercase(),
        "PascalCase" => name.split('_').map(capitalize).collect(),
        "camelCase" => {
            let mut parts = name.split('_');
            let first = parts.next().unwrap_or("").to_lowercase();
            first + &parts.map(capitalize).collect::<String>()
        }
        "snake_case" => name.to_string(),
        "SCREAMING_SNAKE_CASE" => name.to_uppercase(),
        "kebab-case" => name.replace('_', "-"),
        "SCREAMING-KEBAB-CASE" => name.to_uppercase().replace('_', "-"),
        _ => name.to_string(),
    }
}

fn capitalize(w: &str) -> String {
    let mut c = w.chars();
    c.next()
        .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
        .unwrap_or_default()
}

fn pascal_to_snake(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            out.push('_');
        }
        out.extend(c.to_lowercase());
    }
    out
}

fn resolve_name(
    name: &str,
    explicit_rename: Option<&String>,
    rename_all: Option<&str>,
    transform: impl Fn(&str) -> String,
) -> String {
    explicit_rename
        .map(|s| s.to_string())
        .or_else(|| rename_all.map(|rule| apply_rename_all(&transform(name), rule)))
        .unwrap_or_else(|| name.to_string())
}

fn resolve_key(
    field_name: &str,
    field_attrs: &SerdeFieldAttrs,
    rename_all: Option<&str>,
) -> String {
    resolve_name(field_name, field_attrs.rename.as_ref(), rename_all, |n| {
        n.to_string()
    })
}

fn resolve_variant_key(
    variant_name: &str,
    attrs: &SerdeFieldAttrs,
    rename_all: Option<&str>,
) -> String {
    resolve_name(
        variant_name,
        attrs.rename.as_ref(),
        rename_all,
        |n| match rename_all {
            Some("lowercase") => n.to_lowercase(),
            Some("UPPERCASE") => n.to_uppercase(),
            _ => pascal_to_snake(n),
        },
    )
}

fn single_generic_arg<'a>(ty: &'a Type, outer: &str) -> Option<&'a Type> {
    let Type::Path(TypePath { path, qself: None }) = ty else {
        return None;
    };
    let seg = path.segments.last()?;
    if seg.ident != outer {
        return None;
    }
    let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) = &seg.arguments
    else {
        return None;
    };
    if args.len() != 1 {
        return None;
    }
    let GenericArgument::Type(inner) = args.first()? else {
        return None;
    };
    Some(inner)
}
