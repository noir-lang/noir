use std::collections::{HashMap, VecDeque};

use crate::errors::{RuntimeError, RuntimeErrorKind};

use super::{
    context::SsaContext,
    mem::ArrayId,
    node::{NodeId, ObjectType, Opcode, Operation},
};

#[derive(Clone, Debug)]
pub enum MemItem {
    NonConst(NodeId),
    Const(usize),     //represent a bunch of memory instructions having constant index
    ConstLoad(usize), //represent a bunch of load instructions having constant index
}

#[derive(Debug)]
pub enum CseAction {
    ReplaceWith(NodeId),
    Remove(NodeId),
    Keep,
}

/// A list of instructions with the same Operation variant
#[derive(Default, Clone)]
pub struct Anchor {
    map: HashMap<Opcode, HashMap<Operation, NodeId>>, //standard anchor
    cast_map: HashMap<NodeId, HashMap<crate::node::ObjectType, NodeId>>, //cast anchor
    mem_map: HashMap<ArrayId, Vec<VecDeque<(usize, NodeId)>>>, //Memory anchor: one Vec for each array where Vec[i] contains the list of load and store instructions having index i, and the mem_item position in which they appear
    mem_list: HashMap<ArrayId, VecDeque<MemItem>>, // list of the memory instructions, per array, and grouped into MemItems
}

impl Anchor {
    pub fn push_cast_front(&mut self, operation: &Operation, id: NodeId, res_type: ObjectType) {
        match operation {
            Operation::Cast(cast_id) => {
                let mut by_type = HashMap::new();
                by_type.insert(res_type, id);
                self.cast_map.insert(*cast_id, by_type);
            }
            _ => unreachable!(),
        }
    }

    pub fn push_front(&mut self, operation: &Operation, id: NodeId) {
        let op = operation.opcode();

        match operation {
            Operation::Cast(_) => unreachable!(),
            _ => {
                if let std::collections::hash_map::Entry::Vacant(e) = self.map.entry(op) {
                    let mut prev_list = HashMap::new();
                    prev_list.insert(operation.clone(), id);
                    e.insert(prev_list);
                } else {
                    self.map.get_mut(&op).unwrap().insert(operation.clone(), id);
                }
            }
        }
    }

    pub fn find_similar_instruction(&self, operation: &Operation) -> Option<NodeId> {
        let op = operation.opcode();
        self.map.get(&op).and_then(|inner| inner.get(operation).copied())
    }

    pub fn find_similar_cast(
        &self,
        igen: &SsaContext,
        operator: &Operation,
        res_type: super::node::ObjectType,
    ) -> Option<NodeId> {
        match operator {
            Operation::Cast(id) => {
                if self.cast_map.contains_key(id) {
                    let by_type = &self.cast_map[id];
                    if by_type.contains_key(&res_type) {
                        let tu = by_type[&res_type];
                        if let Some(ins) = igen.try_get_instruction(tu) {
                            if !ins.is_deleted() {
                                return Some(tu);
                            }
                        }
                    }
                }
            }
            _ => unreachable!(),
        }

        None
    }

    fn get_mem_map(&self, a: &ArrayId) -> &Vec<VecDeque<(usize, NodeId)>> {
        &self.mem_map[a]
    }

    pub fn get_mem_all(&self, a: ArrayId) -> &VecDeque<MemItem> {
        &self.mem_list[&a]
    }

    pub fn use_array(&mut self, a: ArrayId, len: usize) {
        if !self.mem_list.contains_key(&a) {
            let def: VecDeque<(usize, NodeId)> = VecDeque::new();
            self.mem_list.entry(a).or_insert_with(VecDeque::new);
            self.mem_map.entry(a).or_insert_with(|| vec![def; len]);
        }
    }

    pub fn push_mem_instruction(
        &mut self,
        ctx: &SsaContext,
        id: NodeId,
    ) -> Result<(), RuntimeError> {
        let ins = ctx.get_instruction(id);
        let (array_id, index, is_load) = Anchor::get_mem_op(&ins.operation);
        self.use_array(array_id, ctx.mem[array_id].len as usize);
        let prev_list = self.mem_list.get_mut(&array_id).unwrap();
        let len = prev_list.len();
        if let Some(index_value) = ctx.get_as_constant(index) {
            let mem_idx = index_value.to_u128() as usize;
            let last_item = prev_list.front();
            let item_pos = match last_item {
                Some(MemItem::Const(pos)) => *pos,
                Some(MemItem::ConstLoad(pos)) => {
                    if is_load {
                        *pos
                    } else {
                        let item_pos = pos + 1;
                        prev_list.push_front(MemItem::Const(pos + 1));
                        item_pos
                    }
                }
                None | Some(MemItem::NonConst(_)) => {
                    if is_load {
                        prev_list.push_front(MemItem::ConstLoad(len));
                    } else {
                        prev_list.push_front(MemItem::Const(len));
                    }
                    len
                }
            };
            let anchor_list = self.get_anchor_list_mut(&array_id, mem_idx)?;
            anchor_list.push_front((item_pos, id));
        } else {
            prev_list.push_front(MemItem::NonConst(id));
        }
        Ok(())
    }

    pub fn find_similar_mem_instruction(
        &self,
        ctx: &SsaContext,
        op: &Operation,
        prev_ins: &VecDeque<MemItem>,
    ) -> Result<CseAction, RuntimeErrorKind> {
        for iter in prev_ins.iter() {
            if let Some(action) = self.match_mem_item(ctx, iter, op)? {
                return Ok(action);
            }
        }

        Ok(CseAction::Keep)
    }

    fn get_mem_op(op: &Operation) -> (ArrayId, NodeId, bool) {
        match op {
            Operation::Load { array_id, index } => (*array_id, *index, true),
            Operation::Store { array_id, index, .. } => (*array_id, *index, false),
            _ => unreachable!(),
        }
    }

    fn match_mem_item(
        &self,
        ctx: &SsaContext,
        item: &MemItem,
        op: &Operation,
    ) -> Result<Option<CseAction>, RuntimeErrorKind> {
        let (array_id, index, is_load) = Anchor::get_mem_op(op);
        if let Some(b_value) = ctx.get_as_constant(index) {
            match item {
                MemItem::Const(p) | MemItem::ConstLoad(p) => {
                    let b_idx = b_value.to_u128() as usize;
                    let anchor_list = self.get_anchor_list(&array_id, b_idx)?;
                    for (pos, id) in anchor_list {
                        if pos == p {
                            let action = Anchor::match_mem_id(ctx, *id, index, is_load);
                            if action.is_some() {
                                return Ok(action);
                            }
                        }
                    }

                    Ok(None)
                }
                MemItem::NonConst(id) => Ok(Anchor::match_mem_id(ctx, *id, index, is_load)),
            }
        } else {
            match item {
                MemItem::Const(_) => Ok(Some(CseAction::Keep)),
                MemItem::ConstLoad(_) => {
                    if is_load {
                        Ok(None)
                    } else {
                        Ok(Some(CseAction::Keep))
                    }
                }
                MemItem::NonConst(id) => Ok(Anchor::match_mem_id(ctx, *id, index, is_load)),
            }
        }
    }

    //Returns the anchor list of memory instructions for the array_id at the provided index
    // It issues an out-of-bound error when the list does not exist at this index.
    fn get_anchor_list(
        &self,
        array_id: &ArrayId,
        index: usize,
    ) -> Result<&VecDeque<(usize, NodeId)>, RuntimeErrorKind> {
        let memory_map = self.get_mem_map(array_id);
        memory_map.get(index).ok_or(RuntimeErrorKind::ArrayOutOfBounds {
            index: index as u128,
            bound: memory_map.len() as u128,
        })
    }

    //Same as get_anchor_list() but returns a mutable anchor
    fn get_anchor_list_mut(
        &mut self,
        array_id: &ArrayId,
        index: usize,
    ) -> Result<&mut VecDeque<(usize, NodeId)>, RuntimeErrorKind> {
        let memory_map = self.mem_map.get_mut(&array_id).unwrap();
        let len = memory_map.len() as u128;
        memory_map
            .get_mut(index)
            .ok_or(RuntimeErrorKind::ArrayOutOfBounds { index: index as u128, bound: len })
    }

    fn match_mem_id(
        ctx: &SsaContext,
        a: NodeId,
        b_idx: NodeId,
        b_is_load: bool,
    ) -> Option<CseAction> {
        if b_is_load {
            if let Some(ins_iter) = ctx.try_get_instruction(a) {
                match &ins_iter.operation {
                    Operation::Load { index, .. } => {
                        if !ctx.maybe_distinct(*index, b_idx) {
                            return Some(CseAction::ReplaceWith(a));
                        }
                    }
                    Operation::Store { index, value, .. } => {
                        if !ctx.maybe_distinct(*index, b_idx) {
                            return Some(CseAction::ReplaceWith(*value));
                        }
                        if ctx.maybe_equal(*index, b_idx) {
                            return Some(CseAction::Keep);
                        }
                    }
                    _ => {
                        unreachable!(
                            "invalid operator in the memory anchor list: {:?}",
                            ins_iter.operation
                        )
                    }
                }
            }
        } else if let Some(ins_iter) = ctx.try_get_instruction(a) {
            match ins_iter.operation {
                Operation::Load { index, .. } => {
                    if ctx.maybe_equal(index, b_idx) {
                        return Some(CseAction::Keep);
                    }
                }
                Operation::Store { index, .. } => {
                    if !ctx.maybe_distinct(index, b_idx) {
                        return Some(CseAction::Remove(a));
                    }
                    if ctx.maybe_equal(index, b_idx) {
                        return Some(CseAction::Keep);
                    }
                }
                _ => unreachable!("invalid operator in the memory anchor list"),
            }
        }

        None
    }
}
