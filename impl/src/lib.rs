extern crate proc_macro;

use std::ops::{Add, AddAssign};

use convert_case::Casing;
use darling::{ast::NestedMeta, FromAttributes, FromMeta};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Field, Ident, ItemStruct};

#[derive(Clone, Debug, Default, FromMeta)]
struct BitfieldAttr {
    /// Number of bits to use for the bitfield
    ///
    /// If not specified, the type of the field needs to implement `FieldSpec` trait
    pub bits: Option<u8>,

    /// Default value for the bitfield
    pub default: Option<syn::Expr>,

    /// Endianness of the underlay type
    pub endianness: Option<syn::Expr>,

    /// Use From/Into to convert the value
    #[darling(default)]
    pub from_into: bool,

    /// Closure convert the value from the underlay type
    pub from: Option<syn::Expr>,

    /// Closure convert the value into the underlay type
    pub into: Option<syn::Expr>,
}

impl Add<&BitfieldAttr> for BitfieldAttr {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self::Output {
        Self {
            bits: self.bits.or(rhs.bits),
            default: self.default.or(rhs.default.clone()),
            endianness: self.endianness.or(rhs.endianness.clone()),
            from_into: self.from_into || rhs.from_into,
            from: self.from.or(rhs.from.clone()),
            into: self.into.or(rhs.into.clone()),
        }
    }
}

impl Add for BitfieldAttr {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self + &rhs
    }
}

impl AddAssign for BitfieldAttr {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.clone() + rhs;
    }
}

impl FromAttributes for BitfieldAttr {
    fn from_attributes(attrs: &[syn::Attribute]) -> darling::Result<Self> {
        let mut final_attr = Self::default();

        for attr in attrs {
            if attr.path().is_ident("bitfield") {
                let meta = &attr.meta;
                let attr = Self::from_meta(meta)?;

                final_attr += attr;
            }
        }

        Ok(final_attr)
    }
}

fn gen_field_def(
    field: &Field,
    // global attr, used to set endianness...
    attr: BitfieldAttr,
    bits: Option<u8>,
    mask: u64,
    shift: u8,
    hybrid_field_name: &Option<Ident>,
) -> (
    Ident,
    Ident,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
) {
    let field_name = field.ident.as_ref().unwrap().clone();
    let field_name_mut = format_ident!("{}_mut", field_name);
    let target_type = &field.ty;
    let bitfield_attr = BitfieldAttr::from_attributes(&field.attrs).unwrap() + attr;
    let doc_attr = field.attrs.iter().filter(|a| a.path().is_ident("doc"));

    let field_spec_name = format_ident!(
        "{}Spec",
        field_name
            .to_string()
            .to_case(convert_case::Case::UpperCamel)
    );
    let field_name_uc = format_ident!(
        "{}",
        field_name
            .to_string()
            .to_case(convert_case::Case::UpperCamel)
    );

    let (underlay, mask) = match bits {
        Some(bits) => match bits {
            8 => {
                let mask = mask as u8;
                (quote! { u8 }, quote! { #mask })
            }
            16 => {
                let mask = mask as u16;
                (quote! { u16 }, quote! { #mask as u16 })
            }
            24 => {
                let mask: [u8; 3] = [
                    ((mask >> 16) & 0xff) as u8,
                    ((mask >> 8) & 0xff) as u8,
                    (mask & 0xff) as u8,
                ];
                (quote! { [u8; 3] }, quote! { [#(#mask,)*] })
            }
            32 => {
                let mask = mask as u32;
                (quote! { u32 }, quote! { #mask })
            }
            40 => {
                let mask: [u8; 5] = [
                    ((mask >> 32) & 0xff) as u8,
                    ((mask >> 24) & 0xff) as u8,
                    ((mask >> 16) & 0xff) as u8,
                    ((mask >> 8) & 0xff) as u8,
                    (mask & 0xff) as u8,
                ];
                (quote! { [u8; 5] }, quote! { [#(#mask,)*] })
            }
            48 => {
                let mask: [u8; 6] = [
                    ((mask >> 40) & 0xff) as u8,
                    ((mask >> 32) & 0xff) as u8,
                    ((mask >> 24) & 0xff) as u8,
                    ((mask >> 16) & 0xff) as u8,
                    ((mask >> 8) & 0xff) as u8,
                    (mask & 0xff) as u8,
                ];
                (quote! { [u8; 6] }, quote! { [#(#mask,)*] })
            }
            56 => {
                let mask: [u8; 7] = [
                    ((mask >> 48) & 0xff) as u8,
                    ((mask >> 40) & 0xff) as u8,
                    ((mask >> 32) & 0xff) as u8,
                    ((mask >> 24) & 0xff) as u8,
                    ((mask >> 16) & 0xff) as u8,
                    ((mask >> 8) & 0xff) as u8,
                    (mask & 0xff) as u8,
                ];
                (quote! { [u8; 7] }, quote! { [#(#mask,)*] })
            }
            64 => (quote! { u64 }, quote! { #mask }),
            _ => unreachable!(),
        },
        None => (
            quote! { <#target_type as dmbf::FieldSpec>::Underlay },
            quote! { <#target_type as dmbf::FieldSpec>::MASK },
        ),
    };

    let default_value = bitfield_attr.default;
    let default_value = match default_value {
        Some(default_value) => quote! { #default_value },
        None => quote! { <#underlay as dmbf::FieldSpec>::DEFAULT },
    };

    // let mask = quote! { #mask as Self::Ux };
    let shift = quote! { #shift };

    let endianness = match bitfield_attr.endianness {
        Some(endianness) => quote! { #endianness },
        None => quote! { dmbf::Endianness::Lsb0 },
    };

    let from_inner = if let Some(f) = bitfield_attr.from {
        quote! { (#f)(v) }
    } else if bitfield_attr.from_into {
        quote! { Self::Target::from(v) }
    } else {
        quote! { <Self::Target as dmbf::FieldSpec>::from_underlay(v) }
    };
    let into_inner = if let Some(f) = bitfield_attr.into {
        quote! { (#f)(v) }
    } else if bitfield_attr.from_into {
        quote! { Self::Underlay::from(v) }
    } else {
        quote! { <Self::Target as dmbf::FieldSpec>::into_underlay(v) }
    };

    let field_def = quote! {
        pub struct #field_spec_name;
        impl dmbf::FieldSpec for #field_spec_name {
            type Underlay = #underlay;
            const DEFAULT: Self::Underlay = #default_value;
            const MASK: Self::Underlay = #mask;
            const SHIFT: u8 = #shift;
            const ENDIANNESS: dmbf::Endianness = #endianness;
            type Target = #target_type;
            #[inline]
            fn from_underlay(v: Self::Underlay) -> Self::Target {
                #from_inner
            }
            #[inline]
            fn into_underlay(v: Self::Target) -> Self::Underlay {
                #into_inner
            }
        }
        #(#doc_attr)*
        pub type #field_name_uc = dmbf::Field<#field_spec_name>;
    };

    let field_method = match hybrid_field_name {
        Some(hybrid_field_name) => {
            quote! {
                #[inline]
                pub fn #field_name(&self) -> &#field_name_uc {
                    unsafe { &self.#hybrid_field_name.#field_name }
                }

                #[inline]
                pub fn #field_name_mut(&mut self) -> &mut #field_name_uc {
                    unsafe { &mut self.#hybrid_field_name.#field_name }
                }
            }
        }
        None => {
            quote! {
                #[inline]
                pub fn #field_name(&self) -> &#field_name_uc {
                    &self.#field_name
                }

                #[inline]
                pub fn #field_name_mut(&mut self) -> &mut #field_name_uc {
                    &mut self.#field_name
                }
            }
        }
    };

    (field_name, field_name_uc, field_def, field_method)
}

#[proc_macro_attribute]
pub fn bitfield(attr: TokenStream, input: TokenStream) -> TokenStream {
    let global_attr = NestedMeta::parse_meta_list(attr.into()).unwrap();
    let global_attr = BitfieldAttr::from_list(&global_attr).unwrap();
    let item = parse_macro_input!(input as ItemStruct);

    let attrs = &item.attrs;
    let vis = &item.vis;
    let name = &item.ident;

    // convert the name to snake case to get the name of module
    let mod_name = format_ident!("{}", name.to_string().to_case(convert_case::Case::Snake));

    let mut field_names: Vec<Ident> = Vec::new();
    let mut field_types: Vec<Ident> = Vec::new();
    let mut field_defs: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut field_methods: Vec<proc_macro2::TokenStream> = Vec::new();

    let mut hybrid = false;
    let mut hybrid_field: (Option<Ident>, u8, Vec<(u8, Field)>) = (
        None,   // name
        0,      // bits
        vec![], // subfields
    );

    for field in &item.fields {
        let field_attr = BitfieldAttr::from_attributes(&field.attrs).unwrap();

        // let field_ty = &field.ty;

        if hybrid {
            // Update hybrid field
            let bit = field_attr.bits.unwrap(); // Assume bits is not None
            hybrid_field.0 = hybrid_field
                .0
                .map(|pre| format_ident!("{}_{}", pre, field.ident.as_ref().unwrap()));
            hybrid_field.1 += bit;
            hybrid_field.2.push((bit, field.clone()));

            // Check if the bits can be composed into 8, 16, 32, 64
            if hybrid_field.1 % 8 != 0 {
                // Cannot be composed into 8, 16, 32, 64
                continue;
            } else {
                // Can be composed into 8, 16, 32, 64
                hybrid = false;

                // Generate the hybrid field
                // Step 1: Generate Field<SubfieldSpec> for each subfield
                //         Calculate the mask and shift for each subfield
                // Step 2: Generate HybridField: union of all subfields
                // Step 3: Generate methods for accessing the subfields

                let hybrid_field_name = hybrid_field.0.clone();

                // Step 1
                let mut subfields_names: Vec<Ident> = Vec::new();
                let mut subfields_types: Vec<Ident> = Vec::new();
                let mut prefix_bits: u8 = 0;
                for (b, f) in &hybrid_field.2 {
                    // Calculate the mask and shift
                    // | 0; prefix_bits | 1; b | 0; shift |
                    let shift = hybrid_field.1 - prefix_bits - b;
                    let mask: u64 = ((1 << b) - 1) << shift;
                    prefix_bits += b;

                    let (subfield_name, subfield_type, subfield_def, subfield_methods) =
                        // gen_field_def(f, Some(hybrid_field.1), mask, shift, &hybrid_field_name);
                        gen_field_def(f, global_attr.clone(), Some(hybrid_field.1), mask, shift,&hybrid_field_name);

                    subfields_names.push(subfield_name);
                    subfields_types.push(subfield_type);
                    field_defs.push(subfield_def);
                    field_methods.push(subfield_methods);
                }

                // Step 2
                let hybrid_field_type = format_ident!(
                    "{}",
                    hybrid_field
                        .0
                        .unwrap()
                        .to_string()
                        .to_case(convert_case::Case::UpperCamel)
                );
                let hybrid_field_def = quote! {
                    #[repr(C)]
                    pub union #hybrid_field_type {
                        #(#subfields_names: core::mem::ManuallyDrop<#subfields_types>,)*
                    }
                };

                // Update the vectors
                field_names.push(hybrid_field_name.unwrap());
                field_types.push(hybrid_field_type);
                field_defs.push(hybrid_field_def);

                // Reset the hybrid field
                hybrid_field.0 = None; // name
                hybrid_field.1 = 0; // bits
                hybrid_field.2.clear(); // subfields
            }
        } else {
            // Not hybrid

            // Check if the bits % 8 != 0
            // if so, set hybrid flag
            if let Some(bits) = field_attr.bits {
                if bits % 8 != 0 {
                    hybrid = true;
                    hybrid_field.0 = Some(field.ident.as_ref().unwrap().clone());
                    hybrid_field.1 = bits;
                    hybrid_field.2.push((bits, field.clone()));
                    continue;
                }
            }

            // Generate single field
            let (field_name, field_type, field_def, field_method) =
                // gen_field_def(field, field_attr.bits, !0, 0, &None);
                gen_field_def(field, global_attr.clone(), field_attr.bits, !0, 0,  &None);

            field_names.push(field_name);
            field_types.push(field_type);
            field_defs.push(field_def);
            field_methods.push(field_method);
        }
    }

    quote! {
        pub mod #mod_name{
            use super::*;

            #(#field_defs)*

            #(#attrs)*
            #[repr(C, align(1))]
            pub struct FieldBlock {
                #(#field_names: #field_types,)*
            }

            impl FieldBlock {
                #(#field_methods)*
            }

            pub struct #name<'a> {
                pub data: &'a [u8],
            }

            impl<'a> core::ops::Deref for #name<'a> {
                type Target = FieldBlock;

                fn deref(&self) -> &Self::Target {
                    unsafe { &*(self.data.as_ptr() as *const Self::Target) }
                }
            }

            impl<'a> core::ops::DerefMut for #name<'a> {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    unsafe { &mut *(self.data.as_ptr() as *mut Self::Target) }
                }
            }

            impl<'a, T> From<&'a T> for #name<'a>
            where
                T: AsRef<[u8]>,
            {
                fn from(data: &'a T) -> Self {
                    Self {
                        data: data.as_ref(),
                    }
                }
            }

            impl<'a> From<&'a [u8]> for #name<'a> {
                fn from(data: &'a [u8]) -> Self {
                    Self { data }
                }
            }

            impl<'a> Into<[u8; core::mem::size_of::<FieldBlock>()]> for #name<'a> {
                fn into(self) -> [u8; core::mem::size_of::<FieldBlock>()] {
                    let mut data = [0u8; core::mem::size_of::<FieldBlock>()];
                    data.copy_from_slice(self.data);
                    data
                }
            }
        }
        #vis use #mod_name::#name;
    }
    .into()
}
