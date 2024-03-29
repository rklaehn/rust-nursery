use std::cmp::Ordering;

/// The read part of the merge state that is needed for the binary merge algorithm
/// it just needs random access for the remainder of a and b
///
/// Very often A and B are the same type, but this is not strictly necessary
pub(crate) trait MergeStateRead<A, B> {
    /// The remaining data in a
    fn a_slice(&self) -> &[A];
    /// The remaining data in b
    fn b_slice(&self) -> &[B];
}

/// A binary merge operation
///
/// It is often useful to keep the merge operation and the merge state separate. E.g. computing the
/// intersection and checking if the intersection exists can be done with the same operation, but
/// a different merge state. Likewise in-place operations and operations that produce a new entity
/// can use the same merge operation. THerefore, the merge state is an additional parameter.SortedPairIter
///
/// The operation itself will often be a zero size struct
pub(crate) trait MergeOperation<A, B, M: MergeStateRead<A, B>> {
    fn from_a(&self, m: &mut M, n: usize);
    fn from_b(&self, m: &mut M, n: usize);
    fn collision(&self, m: &mut M);
    fn cmp(&self, a: &A, b: &B) -> Ordering;
    /// merge `an` elements from a and `bn` elements from b into the result
    fn merge0(&self, m: &mut M, an: usize, bn: usize) {
        if an == 0 {
            if bn > 0 {
                self.from_b(m, bn);
            }
        } else if bn == 0 {
            if an > 0 {
                self.from_a(m, an);
            }
        } else {
            // neither a nor b are 0
            let am: usize = an / 2;
            // pick the center element of a and find the corresponding one in b using binary search
            let a = &m.a_slice()[am];
            match m.b_slice()[..bn].binary_search_by(|b| self.cmp(a, b).reverse()) {
                Ok(bm) => {
                    // same elements. bm is the index corresponding to am
                    // merge everything below am with everything below the found element bm
                    self.merge0(m, am, bm);
                    // add the elements a(am) and b(bm)
                    self.collision(m);
                    // merge everything above a(am) with everything above the found element
                    self.merge0(m, an - am - 1, bn - bm - 1);
                }
                Err(bi) => {
                    // not found. bi is the insertion point
                    // merge everything below a(am) with everything below the found insertion point bi
                    self.merge0(m, am, bi);
                    // add a(am)
                    self.from_a(m, 1);
                    // everything above a(am) with everything above the found insertion point
                    self.merge0(m, an - am - 1, bn - bi);
                }
            }
        }
    }
    fn merge(&self, m: &mut M) {
        let a1 = m.a_slice().len();
        let b1 = m.b_slice().len();
        self.merge0(m, a1, b1);
    }
}

/// Basically a convenient to use bool to allow aborting a piece of code early using ?
/// return `None` to abort and `Some(())` to continue
pub(crate) type EarlyOut = Option<()>;

/// This is exactly the same as MergeOperation, except that it allows aborting the operation early.
/// In theory we could have just this operation with no runtime cost, since rust/LLVM will optimize away
/// the EarlyOut when not used. But it is convenient to have two versions.
pub(crate) trait ShortcutMergeOperation<A, B, M: MergeStateRead<A, B>> {
    fn from_a(&self, m: &mut M, n: usize) -> EarlyOut;
    fn from_b(&self, m: &mut M, n: usize) -> EarlyOut;
    fn collision(&self, m: &mut M) -> EarlyOut;
    fn cmp(&self, a: &A, b: &B) -> Ordering;
    /// merge `an` elements from a and `bn` elements from b into the result
    fn merge0(&self, m: &mut M, an: usize, bn: usize) -> EarlyOut {
        if an == 0 {
            if bn > 0 {
                self.from_b(m, bn)?
            }
        } else if bn == 0 {
            if an > 0 {
                self.from_a(m, an)?
            }
        } else {
            // neither a nor b are 0
            let am: usize = an / 2;
            // pick the center element of a and find the corresponding one in b using binary search
            let a = &m.a_slice()[am];
            match m.b_slice()[..bn].binary_search_by(|b| self.cmp(a, b).reverse()) {
                Ok(bm) => {
                    // same elements. bm is the index corresponding to am
                    // merge everything below am with everything below the found element bm
                    self.merge0(m, am, bm)?;
                    // add the elements a(am) and b(bm)
                    self.collision(m)?;
                    // merge everything above a(am) with everything above the found element
                    self.merge0(m, an - am - 1, bn - bm - 1)?;
                }
                Err(bi) => {
                    // not found. bi is the insertion point
                    // merge everything below a(am) with everything below the found insertion point bi
                    self.merge0(m, am, bi)?;
                    // add a(am)
                    self.from_a(m, 1)?;
                    // everything above a(am) with everything above the found insertion point
                    self.merge0(m, an - am - 1, bn - bi)?;
                }
            }
        }
        Some(())
    }
    fn merge(&self, m: &mut M) {
        let a1 = m.a_slice().len();
        let b1 = m.b_slice().len();
        self.merge0(m, a1, b1);
    }
}
