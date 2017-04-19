extern crate gemm_oxide;
extern crate core;
extern crate typenum;
extern crate hwloc;
extern crate libc;

use std::time::{Instant};
use typenum::{U1};

//use gemm_oxide::kern::hsw::{Ukernel, KernelMN, KernelNM, GemvAL1};
use gemm_oxide::kern::snb;
use gemm_oxide::matrix::{Mat, ColumnPanelMatrix, RowPanelMatrix, Matrix, Hierarch};
use gemm_oxide::composables::{GemmNode, PartM, PartN, PartK, PackA, PackB, SpawnThreads, ParallelN, TheRest};
use gemm_oxide::thread_comm::ThreadInfo;
use gemm_oxide::util;



fn compare_packing() {
    type KC = typenum::U256; 
    type MC = typenum::U96; 
    type NR = typenum::U4;
    type MR = typenum::U8;
    type HierA<T> = Hierarch<T, MR, KC, U1, MR>;
    type HierB<T> = Hierarch<T, KC, NR, NR, U1>;
    type HierC<T> = Hierarch<T, MR, NR, NR, U1>;

    type MTAPH<T> = Hierarch<T, MR, KC, U1, MR>;
    type MTBPH<T> = Hierarch<T, KC, NR, NR, U1>;
    type GotoH<T,MTA,MTB,MTC> 
        = SpawnThreads<T, MTA, MTB, MTC,
          PartK<T, MTA, MTB, MTC, KC,
          PackB<T, MTA, MTB, MTC, MTBPH<T>,
          PartM<T, MTA, MTBPH<T>, MTC, MC,
          PackA<T, MTA, MTBPH<T>, MTC, MTAPH<T>,
          ParallelN<T, MTAPH<T>, MTBPH<T>, MTC, NR, TheRest,  
          PartN<T, MTAPH<T>, MTBPH<T>, MTC, NR,
          PartM<T, MTAPH<T>, MTBPH<T>, MTC, MR,
          snb::Ukernel<T, MTAPH<T>, MTBPH<T>, MTC>>>>>>>>>;

    type CPanel<T> = ColumnPanelMatrix<T,NR>; 
    type RPanel<T> = RowPanelMatrix<T,MR>; 
    type Goto<T,MTA,MTB,MTC> 
        = SpawnThreads<T, MTA, MTB, MTC,
          PartK<T, MTA, MTB, MTC, KC,
          PackB<T, MTA, MTB, MTC, CPanel<T>,
          PartM<T, MTA, CPanel<T>, MTC, MC,
          PackA<T, MTA, CPanel<T>, MTC, RPanel<T>,
          ParallelN<T, RPanel<T>, CPanel<T>, MTC, NR, TheRest,  
          PartN<T, RPanel<T>, CPanel<T>, MTC, NR,
          PartM<T, RPanel<T>, CPanel<T>, MTC, MR,
          snb::Ukernel<T, RPanel<T>, CPanel<T>, MTC>>>>>>>>>;

    type GotoOrig  = Goto<f64, Matrix<f64>, Matrix<f64>, Matrix<f64>>;
    type GotoHier  = GotoH<f64, Matrix<f64>, Matrix<f64>, Matrix<f64>>;

    let mut goto  : GotoOrig = GotoOrig::new();
    let mut gotoh : GotoHier = GotoHier::new();
    goto.set_n_threads(2);
    gotoh.set_n_threads(2);
    //let algo_desc = GotoHier::hierarchy_description();
    
    util::pin_to_core(0);

    for index in 1..128 {
        let mut best_time: f64 = 9999999999.0;
        let mut best_time_2: f64 = 9999999999.0;
        let mut worst_err: f64 = 0.0;
        let mut worst_err_2: f64 = 0.0;
        let size = index*8;
        let (m, n, k) = (size, size, size);


        let n_reps = 5;
        for _ in 0..n_reps {
            let mut a : Matrix<f64> = Matrix::new(m, k);
            let mut b : Matrix<f64> = Matrix::new(k, n);
            let mut c : Matrix<f64> = Matrix::new(m, n);
            a.fill_rand(); b.fill_rand(); c.fill_zero();
            c.transpose();

            let mut start = Instant::now();
            unsafe{
                goto.run( &mut a, &mut b, &mut c, &ThreadInfo::single_thread() );
            }
            best_time = best_time.min(util::dur_seconds(start));
            let err = util::test_c_eq_a_b( &mut a, &mut b, &mut c);
            worst_err = worst_err.max(err);

            c.fill_zero();           
            start = Instant::now();
            unsafe{
                gotoh.run( &mut a, &mut b, &mut c, &ThreadInfo::single_thread() );
            }
            best_time_2 = best_time_2.min(util::dur_seconds(start));
            let err = util::test_c_eq_a_b( &mut a, &mut b, &mut c);
            worst_err_2 = worst_err_2.max(err);
        }
        println!("{}\t{}\t{}\t{}\t{}\t{}\t{}", 
                 m, n, k,
                 util::gflops(m,n,k,best_time), 
                 util::gflops(m,n,k,best_time_2), 
                 format!("{:e}", worst_err.sqrt()),
                 format!("{:e}", worst_err_2.sqrt()));

    }
}

fn main() {
    compare_packing();
}
