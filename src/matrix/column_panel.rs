extern crate alloc;

use thread_comm::ThreadInfo;
use typenum::Unsigned;
use self::alloc::heap;
use matrix::{Scalar,Mat,ResizableBuffer,RoCM};
use super::view::{MatrixView};

use core::marker::PhantomData;
use core::{mem};
use core::ptr::{self};
use composables::{AlgorithmStep};

pub struct ColumnPanelMatrix<T: Scalar, PW: Unsigned> {
    y_views: Vec<MatrixView>,
    x_views: Vec<MatrixView>, //offset is in # of panels

    panel_stride: usize,
    buffer: *mut T,
    capacity: usize,
    is_alias: bool,

    _pwt: PhantomData<PW>,
}
impl<T: Scalar, PW: Unsigned> ColumnPanelMatrix<T,PW> {
    pub fn new( h: usize, w: usize ) -> ColumnPanelMatrix<T,PW> {
        assert!(mem::size_of::<T>() != 0, "Matrix can't handle ZSTs");
    
        //Figure out the number of panels
        let panel_w = PW::to_usize();
        let mut n_panels = w / panel_w;
        if n_panels * panel_w < w {
            n_panels += 1;
        }
        let capacity = (n_panels+1) * panel_w * h; //Extra panel for ``preloading'' in ukernel

        let mut y_views : Vec<MatrixView> = Vec::with_capacity( 16 );
        let mut x_views : Vec<MatrixView> = Vec::with_capacity( 16 );
        y_views.push(MatrixView{ offset: 0, padding: 0, iter_size: h }); 
        x_views.push(MatrixView{ offset: 0, padding: 0, iter_size: w }); 

        unsafe { 
            let ptr = heap::allocate( capacity * mem::size_of::<T>(), 4096 );

            ColumnPanelMatrix{ y_views: y_views, x_views: x_views,
                               panel_stride: panel_w*h, 
                               buffer: ptr as *mut _, capacity: capacity,
                               is_alias: false,
                               _pwt: PhantomData }
        }
    }
    

    #[inline(always)]
    pub fn get_panel_stride( &self ) -> usize { self.panel_stride }
/*
    #[inline(always)]
    pub unsafe fn get_buffer( &self ) -> *const T { 
        let panel_w = PW::to_usize();
        let y_view = self.y_views.last().unwrap();
        let x_view = self.x_views.last().unwrap();

        self.buffer.offset((x_view.offset*self.panel_stride + y_view.offset*panel_w) as isize)
    }

    #[inline(always)]
    pub unsafe fn get_mut_buffer( &mut self ) -> *mut T {
        let panel_w = PW::to_usize();
        let y_view = self.y_views.last().unwrap();
        let x_view = self.x_views.last().unwrap();

        self.buffer.offset((x_view.offset*self.panel_stride + y_view.offset*panel_w) as isize)
    }*/

    #[inline(always)]
    pub unsafe fn get_panel( &mut self, id: usize ) -> *mut T {
        let x_view = self.x_views.last().unwrap();
        self.buffer.offset(((x_view.offset + id)*self.panel_stride) as isize)
    }
}
impl<T: Scalar, PW: Unsigned> Mat<T> for ColumnPanelMatrix<T, PW> {
    #[inline(always)]
    fn get( &self, y: usize, x: usize ) -> T {
        let panel_w = PW::to_usize();
        let y_view = self.y_views.last().unwrap();
        let x_view = self.x_views.last().unwrap();

        let panel_id = x / panel_w;
        let panel_index  = x % panel_w;
        let elem_index = (panel_id + x_view.offset) * self.panel_stride + (y + y_view.offset) * panel_w + panel_index;
        unsafe{
            ptr::read( self.buffer.offset(elem_index as isize ) )
        }
    }
    #[inline(always)]
    fn set( &mut self, y: usize, x: usize, alpha: T) {
        let panel_w = PW::to_usize();
        let y_view = self.y_views.last().unwrap();
        let x_view = self.x_views.last().unwrap();

        let panel_id = x / panel_w;
        let panel_index  = x % panel_w;
        let elem_index = (panel_id + x_view.offset) * self.panel_stride + (y + y_view.offset) * panel_w + panel_index;
        unsafe{
            ptr::write( self.buffer.offset(elem_index as isize ), alpha );
        }
    }
    
    #[inline(always)]
    fn off_y( &self ) -> usize { 
        self.y_views.last().unwrap().offset
    }
    #[inline(always)]
    fn off_x( &self ) -> usize { 
        self.x_views.last().unwrap().offset * PW::to_usize()
    }

    #[inline(always)]
    fn set_off_y( &mut self, off_y: usize ) {
        let mut y_view = self.y_views.last_mut().unwrap();
        y_view.offset = off_y;
    }
    #[inline(always)]
    fn set_off_x( &mut self, off_x: usize ) { 
        let panel_w = PW::to_usize();
        if off_x % panel_w != 0 {
            println!("{} {}", off_x, panel_w);
            panic!("Illegal partitioning within ColumnPanelMatrix!");
        }

        let mut x_view = self.x_views.last_mut().unwrap();
        x_view.offset = off_x / panel_w;
    }
    #[inline(always)]
    fn add_off_y( &mut self, start: usize ) { 
        let off = self.off_y();
        self.set_off_y(start + off);
    }   
    #[inline(always)]
    fn add_off_x( &mut self, start: usize ) { 
        let off = self.off_x();
        self.set_off_x(start + off);
    } 


    #[inline(always)]
    fn iter_height( &self ) -> usize {
        let y_view = self.y_views.last().unwrap();
        y_view.iter_size
    }
    #[inline(always)]
    fn iter_width( &self ) -> usize {
        let x_view = self.x_views.last().unwrap();
        x_view.iter_size
    }
    #[inline(always)]
    fn set_iter_height( &mut self, iter_h: usize ) {
        let mut y_view = self.y_views.last_mut().unwrap();
        y_view.iter_size = iter_h;
    }
    #[inline(always)]
    fn set_iter_width( &mut self, iter_w: usize ) {
        let mut x_view = self.x_views.last_mut().unwrap();
        x_view.iter_size = iter_w;
    }

    #[inline(always)]
    fn logical_h_padding( &self ) -> usize {
        let y_view = self.y_views.last().unwrap();
        y_view.padding
    }
    #[inline(always)]
    fn logical_w_padding( &self ) -> usize {
        let x_view = self.x_views.last().unwrap();
        x_view.padding
    }
    #[inline(always)]
    fn set_logical_h_padding( &mut self, h_pad: usize ) {
        let mut y_view = self.y_views.last_mut().unwrap();
        y_view.padding = h_pad
    }
    #[inline(always)]
    fn set_logical_w_padding( &mut self, w_pad: usize ) {
        let mut x_view = self.x_views.last_mut().unwrap();
        x_view.padding = w_pad
    }

    
    fn push_x_view( &mut self, blksz: usize ) -> usize {
        let (zoomed_view, uz_iter_size) = { 
            let uz_view = self.x_views.last().unwrap();
            let (z_iter_size, z_padding) = uz_view.zoomed_size_and_padding(0, blksz);
            (MatrixView{ offset: uz_view.offset, padding: z_padding, iter_size: z_iter_size }, uz_view.iter_size)
        };  
        self.x_views.push(zoomed_view);
        uz_iter_size
    }   
    
    fn push_y_view( &mut self, blksz: usize ) -> usize{
        let (zoomed_view, uz_iter_size) = { 
            let uz_view = self.y_views.last().unwrap();
            let (z_iter_size, z_padding) = uz_view.zoomed_size_and_padding(0, blksz);
            (MatrixView{ offset: uz_view.offset, padding: z_padding, iter_size: z_iter_size }, uz_view.iter_size)
        };  
        self.y_views.push(zoomed_view);
        uz_iter_size
    }

    #[inline(always)]
    fn pop_x_view( &mut self ) {
        debug_assert!( self.x_views.len() >= 2 );
        self.x_views.pop();
    }
    #[inline(always)]
    fn pop_y_view( &mut self ) {
        debug_assert!( self.y_views.len() >= 2 );
        self.y_views.pop();
    }
    
    fn slide_x_view_to( &mut self, x: usize, blksz: usize ) { 
        let view_len = self.x_views.len();
        debug_assert!( view_len >= 2 );

        let uz_view = self.x_views[view_len-2];
        let(z_iter_size, z_padding) = uz_view.zoomed_size_and_padding(x, blksz);

        let mut z_view = self.x_views.last_mut().unwrap();
        z_view.iter_size = z_iter_size;
        z_view.padding = z_padding;
        z_view.offset = uz_view.offset + x / PW::to_usize();
    }   
    
    fn slide_y_view_to( &mut self, y: usize, blksz: usize ) { 
        let view_len = self.y_views.len();
        debug_assert!( view_len >= 2 );

        let uz_view = self.y_views[view_len-2];
        let(z_iter_size, z_padding) = uz_view.zoomed_size_and_padding(y, blksz);

        let mut z_view = self.y_views.last_mut().unwrap();
        z_view.iter_size = z_iter_size;
        z_view.padding = z_padding;
        z_view.offset = uz_view.offset + y;
    }  


    #[inline(always)]
    unsafe fn make_alias( &self ) -> Self {
        let x_view = self.x_views.last().unwrap();
        let y_view = self.y_views.last().unwrap();

        let mut x_views_alias : Vec<MatrixView> = Vec::with_capacity( 16 );
        let mut y_views_alias : Vec<MatrixView> = Vec::with_capacity( 16 );
        x_views_alias.push(MatrixView{ offset: x_view.offset, padding: x_view.offset, iter_size: x_view.iter_size });
        y_views_alias.push(MatrixView{ offset: y_view.offset, padding: y_view.offset, iter_size: y_view.iter_size });

        ColumnPanelMatrix{ y_views: y_views_alias, x_views: x_views_alias,
                           panel_stride: self.panel_stride,
                           buffer: self.buffer, 
                           capacity: self.capacity,
                           is_alias: true,
                           _pwt: PhantomData }
    }

    #[inline(always)]
    unsafe fn send_alias( &mut self, thr: &ThreadInfo<T> ) {
        let buf = thr.broadcast( self.buffer );
        self.is_alias = true;
        self.buffer = buf;
    }
}
impl<T:Scalar, PW: Unsigned> Drop for ColumnPanelMatrix<T, PW> {
    fn drop(&mut self) {
        if !self.is_alias {
            unsafe {
                heap::deallocate(self.buffer as *mut _, mem::size_of::<T>() * self.capacity, 4096);
            }
        }
    }
}

unsafe impl<T:Scalar, PW: Unsigned> Send for ColumnPanelMatrix<T, PW> {}

impl<T:Scalar, PW: Unsigned> ResizableBuffer<T> for ColumnPanelMatrix<T, PW> {
    #[inline(always)]
    fn empty(_: AlgorithmStep, _: AlgorithmStep, _: &Vec<AlgorithmStep>) -> Self {
        ColumnPanelMatrix::new(0,0)
    }
    #[inline(always)]
    fn capacity(&self) -> usize { self.capacity }
    #[inline(always)]
    fn set_capacity(&mut self, capacity: usize) { self.capacity = capacity; }
    #[inline(always)]
    fn capacity_for(other: &Mat<T>, _: AlgorithmStep, _: AlgorithmStep, _: &Vec<AlgorithmStep>) -> usize {
        if other.height() <= 0 || other.width() <= 0 { 
            0   
        } else {
            let new_n_panels = (other.width()-1) / PW::to_usize() + 1;
            (new_n_panels + 1) * PW::to_usize() * other.height()
        }
    }
    #[inline(always)]
    fn aquire_buffer_for(&mut self, req_capacity: usize) {
        let req_padded_capacity = req_capacity;
        if req_padded_capacity > self.capacity {
            unsafe {
                heap::deallocate(self.buffer as *mut _, mem::size_of::<T>() * self.capacity, 4096);
                self.buffer = heap::allocate( req_padded_capacity * mem::size_of::<T>(), 4096 ) as *mut _;
                self.capacity = req_padded_capacity;
            }
        }
    }
    #[inline(always)]
    fn resize_to( &mut self, other: &Mat<T>, _: AlgorithmStep, _: AlgorithmStep, _: &Vec<AlgorithmStep> ) {
        debug_assert!( self.y_views.len() == 1, "Can't resize a submatrix!");
        let mut y_view = self.y_views.last_mut().unwrap();
        let mut x_view = self.x_views.last_mut().unwrap();

        y_view.iter_size = other.iter_height();
        x_view.iter_size = other.iter_width();
        y_view.padding = other.logical_h_padding();
        x_view.padding = other.logical_w_padding();
        self.panel_stride = PW::to_usize()*other.height();
    }
}

impl<T: Scalar, PW: Unsigned> RoCM<T> for ColumnPanelMatrix<T, PW> {
    #[inline(always)]
    fn partition_is_rocm(&self) -> bool { 
        self.width() <= PW::to_usize()
    }

    #[inline(always)]
    fn get_leaf_rs(&self) -> usize { 
        PW::to_usize()
    }

    #[inline(always)]
    fn get_leaf_cs(&self) -> usize { 
        1
    }

    #[inline(always)]
    unsafe fn get_buffer(&self) -> *const T {
        let panel_w = PW::to_usize();
        let y_view = self.y_views.last().unwrap();
        let x_view = self.x_views.last().unwrap();

        self.buffer.offset((x_view.offset*self.panel_stride + y_view.offset*panel_w) as isize)
    }

    #[inline(always)]
    unsafe fn get_mut_buffer( &mut self ) -> *mut T {
        let panel_w = PW::to_usize();
        let y_view = self.y_views.last().unwrap();
        let x_view = self.x_views.last().unwrap();

        self.buffer.offset((x_view.offset*self.panel_stride + y_view.offset*panel_w) as isize)
    }

	#[inline(always)]
    fn get_block_rs(&self, _: usize, blksz: usize) -> usize {
		PW::to_usize()
    }

	#[inline(always)]
    fn get_block_cs(&self, lvl: usize, blksz: usize) -> usize {
        if lvl == 0 {
            1
        } else {
			debug_assert!( blksz % PW::to_usize() == 0 );
			self.panel_stride * blksz / PW::to_usize()
        }
    }
}
