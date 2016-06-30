use matrix::{Scalar,Mat,ColumnPanelMatrix,RowPanelMatrix};
use core::marker::{PhantomData};
use pack::{Copier,Packer};
use thread::{ThreadInfo};

extern crate core;

pub trait GemmNode<T: Scalar, At: Mat<T>, Bt: Mat<T>, Ct: Mat<T>> {
    #[inline(always)]
    unsafe fn run( &mut self, a: &mut At, b: &mut Bt, c: &mut Ct, thr: &ThreadInfo<T> ) -> ();
    unsafe fn shadow( &self ) -> Self where Self: Sized;
}


pub struct PackAcp<T: Scalar, At: Mat<T>, Bt: Mat<T>, Ct: Mat<T>, S: GemmNode<T, ColumnPanelMatrix<T>, Bt, Ct>> {
    child: S,
    _panel_width: usize,
    packer: Packer<T, At, ColumnPanelMatrix<T>>,
    a_pack: ColumnPanelMatrix<T>,
    _t: PhantomData<T>,
    _at: PhantomData<At>,
    _bt: PhantomData<Bt>,
    _ct: PhantomData<Ct>,
} impl<T: Scalar,At: Mat<T>, Bt: Mat<T>, Ct: Mat<T>, S: GemmNode<T, ColumnPanelMatrix<T>, Bt, Ct>> PackAcp <T,At,Bt,Ct,S> {
    pub fn new( panel_width: usize, child: S ) -> PackAcp<T, At, Bt, Ct, S>{
        let matrix = ColumnPanelMatrix::new( 0, 0, panel_width );
        let packer = Packer::new();
        PackAcp{ _panel_width: panel_width, child: child, a_pack: matrix, packer: packer,
                 _t: PhantomData, _at:PhantomData, _bt: PhantomData, _ct: PhantomData }
    }
}
impl<T: Scalar, At: Mat<T>, Bt: Mat<T>, Ct: Mat<T>, S: GemmNode<T, ColumnPanelMatrix<T>, Bt, Ct>>
    GemmNode<T, At, Bt, Ct> for PackAcp<T, At, Bt, Ct, S> {
    #[inline(always)]
    unsafe fn run( &mut self, a: &mut At, b: &mut Bt, c:&mut Ct, thr: &ThreadInfo<T> ) -> () {
        self.a_pack.resize_to_fit( a );
        self.packer.pack( a, &mut self.a_pack );
        self.child.run(&mut self.a_pack, b, c, thr);
    }
    unsafe fn shadow( &self ) -> Self where Self: Sized {
        PackAcp{ _panel_width: self._panel_width, 
                 child: self.child.shadow(), 
                 a_pack: ColumnPanelMatrix::new( 0, 0, self._panel_width), 
                 packer: Packer::new(),
                 _t: PhantomData, _at:PhantomData, _bt: PhantomData, _ct: PhantomData }
    }
}

pub struct PackBcp<T: Scalar, At: Mat<T>, Bt: Mat<T>, Ct: Mat<T>, S: GemmNode<T, At, ColumnPanelMatrix<T>, Ct>> {
    child: S,
    _panel_width: usize,
    packer: Packer<T, Bt, ColumnPanelMatrix<T>>,
    b_pack: ColumnPanelMatrix<T>,
    _t: PhantomData<T>,
    _at: PhantomData<At>,
    _bt: PhantomData<Bt>,
    _ct: PhantomData<Ct>,
}
impl<T: Scalar,At: Mat<T>, Bt: Mat<T>, Ct: Mat<T>, S: GemmNode<T, At, ColumnPanelMatrix<T>, Ct>> 
    PackBcp <T,At,Bt,Ct,S> {
    pub fn new( panel_width: usize, child: S ) -> PackBcp<T, At, Bt, Ct, S>{
        let matrix = ColumnPanelMatrix::new( 0, 0, panel_width );
        let packer = Packer::new();
        PackBcp{ _panel_width: panel_width, child: child, b_pack: matrix, packer: packer,
                 _t: PhantomData, _at:PhantomData, _bt: PhantomData, _ct: PhantomData }
    }
}
impl<T: Scalar, At: Mat<T>, Bt: Mat<T>, Ct: Mat<T>, S: GemmNode<T, At, ColumnPanelMatrix<T>, Ct>>
    GemmNode<T, At, Bt, Ct> for PackBcp<T, At, Bt, Ct, S> {
    #[inline(always)]
    unsafe fn run( &mut self, a: &mut At, b: &mut Bt, c: &mut Ct, thr: &ThreadInfo<T> ) -> () {
        self.b_pack.resize_to_fit( b );
        self.packer.pack( b, &mut self.b_pack );
        self.child.run(a, &mut self.b_pack, c, thr);
    }
    unsafe fn shadow( &self ) -> Self where Self: Sized {
        PackBcp{ _panel_width: self._panel_width, 
                 child: self.child.shadow(), 
                 b_pack: ColumnPanelMatrix::new( 0, 0, self._panel_width), 
                 packer: Packer::new(),
                 _t: PhantomData, _at:PhantomData, _bt: PhantomData, _ct: PhantomData }
    }
}
pub struct PackArp<T: Scalar, At: Mat<T>, Bt: Mat<T>, Ct: Mat<T>, S: GemmNode<T, RowPanelMatrix<T>, Bt, Ct>> {
    child: S,
    _panel_height: usize,
    packer: Packer<T, At, RowPanelMatrix<T>>,
    a_pack: RowPanelMatrix<T>,
    _t: PhantomData<T>,
    _at: PhantomData<At>,
    _bt: PhantomData<Bt>,
    _ct: PhantomData<Ct>,
}
impl<T: Scalar,At: Mat<T>, Bt: Mat<T>, Ct: Mat<T>, S: GemmNode<T, RowPanelMatrix<T>, Bt, Ct>> 
    PackArp <T,At,Bt,Ct,S> {
    pub fn new( panel_height: usize, child: S ) -> PackArp<T, At, Bt, Ct, S>{
        let matrix = RowPanelMatrix::new( 0, 0, panel_height );
        let packer = Packer::new();
        PackArp{ _panel_height: panel_height, child: child, a_pack: matrix, packer: packer,
                 _t: PhantomData, _at:PhantomData, _bt: PhantomData, _ct: PhantomData }
    }
}
impl<T: Scalar, At: Mat<T>, Bt: Mat<T>, Ct: Mat<T>, S: GemmNode<T, RowPanelMatrix<T>, Bt, Ct>>
    GemmNode<T, At, Bt, Ct> for PackArp<T, At, Bt, Ct, S> {
    #[inline(always)]
    unsafe fn run( &mut self, a: &mut At, b: &mut Bt, c: &mut Ct, thr: &ThreadInfo<T> ) -> () {
        self.a_pack.resize_to_fit( a );
        self.packer.pack( a, &mut self.a_pack );
        self.child.run(&mut self.a_pack, b, c, thr);
    }
    unsafe fn shadow( &self ) -> Self where Self: Sized {
        PackArp{ _panel_height: self._panel_height, 
                 child: self.child.shadow(), 
                 a_pack: RowPanelMatrix::new( 0, 0, self._panel_height), 
                 packer: Packer::new(),
                 _t: PhantomData, _at:PhantomData, _bt: PhantomData, _ct: PhantomData }
    }
}

pub struct PackBrp<T: Scalar, At: Mat<T>, Bt: Mat<T>, Ct: Mat<T>, S: GemmNode<T, At, RowPanelMatrix<T>, Ct>> {
    child: S,
    _panel_height: usize,
    packer: Packer<T, Bt, RowPanelMatrix<T>>,
    b_pack: RowPanelMatrix<T>,
    _t: PhantomData<T>,
    _at: PhantomData<At>,
    _bt: PhantomData<Bt>,
    _ct: PhantomData<Ct>,
}
impl<T: Scalar,At: Mat<T>, Bt: Mat<T>, Ct: Mat<T>, S: GemmNode<T, At, RowPanelMatrix<T>, Ct>> 
    PackBrp <T,At,Bt,Ct,S> {
    pub fn new( panel_height: usize, child: S ) -> PackBrp<T, At, Bt, Ct, S>{
        let matrix = RowPanelMatrix::new( 0, 0, panel_height );
        let packer = Packer::new();
        PackBrp{ _panel_height: panel_height, child: child, b_pack: matrix, packer: packer,
                 _t: PhantomData, _at:PhantomData, _bt: PhantomData, _ct: PhantomData }
    }
}
impl<T: Scalar, At: Mat<T>, Bt: Mat<T>, Ct: Mat<T>, S: GemmNode<T, At, RowPanelMatrix<T>, Ct>>
    GemmNode<T, At, Bt, Ct> for PackBrp<T, At, Bt, Ct, S> {
    #[inline(always)]
    unsafe fn run( &mut self, a: &mut At, b: &mut Bt, c: &mut Ct, thr: &ThreadInfo<T> ) -> () {
        self.b_pack.resize_to_fit( b );
        self.packer.pack( b, &mut self.b_pack );
        self.child.run(a, &mut self.b_pack, c, thr);
    }
    unsafe fn shadow( &self ) -> Self where Self: Sized {
        PackBrp{ _panel_height: self._panel_height, 
                 child: self.child.shadow(), 
                 b_pack: RowPanelMatrix::new( 0, 0, self._panel_height), 
                 packer: Packer::new(),
                 _t: PhantomData, _at:PhantomData, _bt: PhantomData, _ct: PhantomData }
    }
}

pub struct PartM<T: Scalar, At: Mat<T>, Bt: Mat<T>, Ct: Mat<T>, S: GemmNode<T, At, Bt, Ct>> {
    bsz: usize,
    child: S,
    _t: PhantomData<T>,
    _at: PhantomData<At>,
    _bt: PhantomData<Bt>,
    _ct: PhantomData<Ct>,
}
impl<T: Scalar, At: Mat<T>, Bt: Mat<T>, Ct: Mat<T>, S: GemmNode<T, At, Bt, Ct>> PartM<T,At,Bt,Ct,S> {
    pub fn new( bsz: usize, child: S ) -> PartM<T, At, Bt, Ct,S>{
            PartM{ bsz: bsz, child: child, 
                   _t: PhantomData, _at: PhantomData, _bt: PhantomData, _ct: PhantomData }
    }
}
impl<T: Scalar, At: Mat<T>, Bt: Mat<T>, Ct: Mat<T>, S: GemmNode<T, At, Bt, Ct>>
    GemmNode<T, At, Bt, Ct> for PartM<T,At,Bt,Ct,S> {
    #[inline(always)]
    unsafe fn run( &mut self, a: &mut At, b: &mut Bt, c: &mut Ct, thr: &ThreadInfo<T> ) -> () {
        let m_save = c.iter_height();
        let ay_off_save = a.off_y();
        let cy_off_save = c.off_y();
        
        let mut i = 0;
        while i < m_save  {
            a.adjust_y_view( m_save, ay_off_save, self.bsz, i);
            c.adjust_y_view( m_save, cy_off_save, self.bsz, i);

            self.child.run(a, b, c, thr);
            i += self.bsz;
        }

        a.set_iter_height( m_save );
        a.set_off_y( ay_off_save );
        c.set_iter_height( m_save );
        c.set_off_y( cy_off_save );
    }
    unsafe fn shadow( &self ) -> Self where Self: Sized {
        PartM{ bsz: self.bsz, child: self.child.shadow(), 
               _t: PhantomData, _at: PhantomData, _bt: PhantomData, _ct: PhantomData }
    }
}

pub struct PartN<T: Scalar, At: Mat<T>, Bt: Mat<T>, Ct: Mat<T>, S: GemmNode<T, At, Bt, Ct>> {
    bsz: usize,
    child: S,
    _t: PhantomData<T>,
    _at: PhantomData<At>,
    _bt: PhantomData<Bt>,
    _ct: PhantomData<Ct>,
}
impl<T: Scalar, At: Mat<T>, Bt: Mat<T>, Ct: Mat<T>, S: GemmNode<T, At, Bt, Ct>> PartN<T,At,Bt,Ct,S> {
    pub fn new( bsz: usize, child: S ) -> PartN<T, At, Bt, Ct, S>{
            PartN{ bsz: bsz, child: child, 
                   _t: PhantomData, _at: PhantomData, _bt: PhantomData, _ct: PhantomData }
    }
}
impl<T: Scalar, At: Mat<T>, Bt: Mat<T>, Ct: Mat<T>, S: GemmNode<T, At, Bt, Ct>>
    GemmNode<T, At, Bt, Ct> for PartN<T,At,Bt,Ct,S> {
    #[inline(always)]
    unsafe fn run( &mut self, a: &mut At, b: &mut Bt, c: &mut Ct, thr:&ThreadInfo<T> ) -> () {
        let n_save = c.iter_width();
        let bx_off_save = b.off_x();
        let cx_off_save = c.off_x();
        
        let mut i = 0;
        while i < n_save {
            b.adjust_x_view( n_save, bx_off_save, self.bsz, i);
            c.adjust_x_view( n_save, cx_off_save, self.bsz, i);

            self.child.run(a, b, c, thr);
            i += self.bsz;
        }

        b.set_iter_width( n_save );
        b.set_off_x( bx_off_save );
        c.set_iter_width( n_save );
        c.set_off_x( cx_off_save );
    }
    unsafe fn shadow( &self ) -> Self where Self: Sized {
        PartN{ bsz: self.bsz, child: self.child.shadow(), 
               _t: PhantomData, _at: PhantomData, _bt: PhantomData, _ct: PhantomData }
    }
}

pub struct PartK<T: Scalar, At: Mat<T>, Bt: Mat<T>, Ct: Mat<T>, S: GemmNode<T, At, Bt, Ct>> {
    bsz: usize,
    child: S,
    _t: PhantomData<T>,
    _at: PhantomData<At>,
    _bt: PhantomData<Bt>,
    _ct: PhantomData<Ct>,
}
impl<T: Scalar, At: Mat<T>, Bt: Mat<T>, Ct: Mat<T>, S: GemmNode<T, At, Bt, Ct>> PartK<T,At,Bt,Ct,S> {
    pub fn new( bsz: usize, child: S ) -> PartK<T, At, Bt, Ct, S>{
        PartK{ bsz: bsz, child: child, 
               _t: PhantomData, _at: PhantomData, _bt: PhantomData, _ct: PhantomData }
    }
}
impl<T: Scalar, At: Mat<T>, Bt: Mat<T>, Ct: Mat<T>, S: GemmNode<T, At, Bt, Ct>>
    GemmNode<T, At, Bt, Ct> for PartK<T,At,Bt,Ct,S> {
    #[inline(always)]
    unsafe fn run( &mut self, a: &mut At, b: &mut Bt, c: &mut Ct, thr: &ThreadInfo<T> ) -> () {
        let k_save = a.iter_width();
        let ax_off_save = a.off_x();
        let by_off_save = b.off_y();
        
        let mut i = 0;
        while i < k_save  {
            a.adjust_x_view( k_save, ax_off_save, self.bsz, i);
            b.adjust_y_view( k_save, by_off_save, self.bsz, i);

            self.child.run(a, b, c, thr);
            i += self.bsz;
        }

        a.set_iter_width( k_save );
        a.set_off_x( ax_off_save );
        b.set_iter_height( k_save );
        b.set_off_y( by_off_save );
    }
    unsafe fn shadow( &self ) -> Self where Self: Sized {
        PartK{ bsz: self.bsz, child: self.child.shadow(), 
               _t: PhantomData, _at: PhantomData, _bt: PhantomData, _ct: PhantomData }
    }
}

pub struct TripleLoopKernel{}
impl TripleLoopKernel {
    pub fn new() -> TripleLoopKernel {
        TripleLoopKernel{}
    }
}
impl<T: Scalar, At: Mat<T>, Bt: Mat<T>, Ct: Mat<T>> 
    GemmNode<T, At, Bt, Ct> for TripleLoopKernel {
    #[inline(always)]
    unsafe fn run( &mut self, a: &mut At, b: &mut Bt, c: &mut Ct, _thr: &ThreadInfo<T> ) -> () {
        //For now, let's do an axpy based gemm
        for x in 0..c.width() {
            for z in 0..a.width() {
                for y in 0..c.height() {
                    let t = a.get(y,z) * b.get(z,x) + c.get(y,x);
                    c.set( y, x, t );
                }
            }
        }
    }
    unsafe fn shadow( &self ) -> Self where Self: Sized {
        TripleLoopKernel{}
    }
}
