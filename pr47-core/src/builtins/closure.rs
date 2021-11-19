use std::any::TypeId;
use std::ptr::NonNull;

use smallvec::SmallVec;
use xjbutil::void::Void;
use crate::builtins::test_container::GenericTestContainer;

use crate::data::Value;
use crate::data::generic::GenericTypeVT;
use crate::data::traits::{ChildrenType, StaticBase};
use crate::data::tyck::{TyckInfo, TyckInfoPool};

pub struct Closure {
    pub capture: SmallVec<[Value; 4]>,
    pub func_id: usize
}

impl Closure {
    pub fn new(capture: SmallVec<[Value; 4]>, func_id: usize) -> Self {
        Self { capture, func_id }
    }
}

impl StaticBase<Closure> for Void {
    fn tyck_info(tyck_info_pool: &mut TyckInfoPool) -> NonNull<TyckInfo> {
        tyck_info_pool.create_container_type(TypeId::of::<Closure>(), &[])
    }

    fn type_name() -> String { "closure".into() }

    #[inline] fn children(vself: *const Closure) -> ChildrenType {
        let vself: &Closure = unsafe { &*vself };
        Some(Box::new(vself.capture.iter().map(|v: &Value| unsafe { v.ptr_repr })))
    }
}

pub fn create_closure_vt(
    tyck_info_pool: &mut TyckInfoPool,
    arg_types: &[NonNull<TyckInfo>]
) -> GenericTypeVT {
    let tyck_info: NonNull<TyckInfo> =
        tyck_info_pool.create_container_type(TypeId::of::<Closure>(), arg_types);

    use crate::data::generic::gen_impls;
    GenericTypeVT {
        tyck_info: unsafe { tyck_info.as_ref().get_container_tyck_info_unchecked() },
        type_name: "closure".to_string(),
        #[cfg(debug_assertions)]
        move_out_fn: gen_impls::generic_move_out_ck::<Closure>,
        #[cfg(not(debug_assertions))]
        move_out_fn: gen_impls::generic_move_out::<Closure>,
        children_fn: gen_impls::generic_children::<Closure>,
        drop_fn: gen_impls::generic_drop::<GenericTestContainer>
    }
}