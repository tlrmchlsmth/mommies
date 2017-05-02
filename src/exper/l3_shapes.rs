#![feature(specialization)]
#![feature(conservative_impl_trait)]
#![feature(cfg_target_feature)]
#![feature(asm)] 

#![allow(unused_imports)]

extern crate core;
extern crate typenum;
extern crate momms;

use std::time::{Instant};
use typenum::{U1, B0, UInt, Unsigned};

use momms::kern::{KernelNM,KernelMN};
use momms::matrix::{Scalar, Mat, ColumnPanelMatrix, RowPanelMatrix, Matrix, Hierarch};
use momms::composables::{GemmNode, AlgorithmStep, PartM, PartN, PartK, PackA, PackB, SpawnThreads, ParallelM, ParallelN, TheRest};
use momms::thread_comm::ThreadInfo;
use momms::util;

fn test_algorithm<T: Scalar, Mr: Unsigned, Nr: Unsigned, Kc:Unsigned, CLRS: Unsigned, CLCS: Unsigned, 
    S: GemmNode<T, Hierarch<T, Mr, Kc, U1, Mr>, Hierarch<T, Kc, Nr, Nr, U1>, Hierarch<T, Mr, Nr, CLRS, CLCS>>>
    ( m:usize, n: usize, k: usize, algo: &mut S, flusher: &mut Vec<f64>, n_reps: usize ) -> (f64, T) 
{
    let algo_desc = S::hierarchy_description();
    let mut best_time: f64 = 9999999999.0;
    let mut worst_err: T = T::zero();

    for _ in 0..n_reps {
        //Create matrices.
        let mut a : Hierarch<T, Mr, Kc, U1, Mr> = Hierarch::new(m, k, &algo_desc, AlgorithmStep::M{bsz: 0}, AlgorithmStep::K{bsz: 0});
        let mut b : Hierarch<T, Kc, Nr, Nr, U1> = Hierarch::new(k, n, &algo_desc, AlgorithmStep::K{bsz: 0}, AlgorithmStep::N{bsz: 0});
        let mut c : Hierarch<T, Mr, Nr, CLRS, CLCS> = Hierarch::new(m, n, &algo_desc, AlgorithmStep::M{bsz: 0}, AlgorithmStep::N{bsz: 0});

        //Fill the matrices
        a.fill_rand(); c.fill_zero(); b.fill_rand();

        //Read a buffer so that A, B, and C are cold in cache.
        for i in flusher.iter_mut() { *i += 1.0; }

        //Time and run algorithm
        let start = Instant::now();
        unsafe{ algo.run( &mut a, &mut b, &mut c, &ThreadInfo::single_thread() ); }
        best_time = best_time.min(util::dur_seconds(start));
        let err = util::test_c_eq_a_b( &mut a, &mut b, &mut c);
        worst_err = worst_err.max(err);
    }
    (best_time, worst_err)
}

fn test(m_selector: isize, n_selector: isize, k_selector: isize) {
    //Goto's algorithm
    type U3000 = UInt<UInt<typenum::U750, B0>, B0>;
    type Nc = U3000;
    type Kc = typenum::U192; 
    type Mc = typenum::U120; 
    type Mr = typenum::U4;
    type Nr = typenum::U12;

    type Goto<T,MTA,MTB,MTC> 
        = SpawnThreads<T, MTA, MTB, MTC,
          PartN<T, MTA, MTB, MTC, Nc,
          PartK<T, MTA, MTB, MTC, Kc,
          PartM<T, MTA, MTB, MTC, Mc,
          ParallelN<T, MTA, MTB, MTC, Nr, TheRest,
          KernelNM<T, MTA, MTB, MTC, Nr, Mr>>>>>>;
    
    type RootS3 = typenum::U768;
    //Resident A algorithm
    type NcL2 = typenum::U120;
    type L3A<T,MTA,MTB,MTC> 
        = SpawnThreads<T, MTA, MTB, MTC,
          PartM<T, MTA, MTB, MTC, RootS3,
          PartK<T, MTA, MTB, MTC, RootS3,
          PartN<T, MTA, MTB, MTC, NcL2,
          PartK<T, MTA, MTB, MTC, Kc,
          ParallelM<T, MTA, MTB, MTC, Nr, TheRest, //This algorithm uses a 12x4 ukernel instead of 4x12 like the other two
          KernelMN<T, MTA, MTB, MTC, Nr, Mr>>>>>>>;

    //Resident B algorithm
    type McL2 = typenum::U120;
    type L3B<T,MTA,MTB,MTC> 
        = SpawnThreads<T, MTA, MTB, MTC,
          PartN<T, MTA, MTB, MTC, RootS3,
          PartK<T, MTA, MTB, MTC, RootS3,
          PartM<T, MTA, MTB, MTC, McL2,
          PartK<T, MTA, MTB, MTC, Kc,
          ParallelN<T, MTA, MTB, MTC, Nr, TheRest,
          KernelNM<T, MTA, MTB, MTC, Nr, Mr>>>>>>>;

    //Resident C algorithm
    type L3C<T,MTA,MTB,MTC> 
        = SpawnThreads<T, MTA, MTB, MTC,
          PartN<T, MTA, MTB, MTC, RootS3,
          PartM<T, MTA, MTB, MTC, typenum::U720, //Use 720 as blocksize as it is divisible by Mc=120
          PartK<T, MTA, MTB, MTC, Kc,
          PartM<T, MTA, MTB, MTC, McL2,
          ParallelN<T, MTA, MTB, MTC, Nr, TheRest,
          KernelNM<T, MTA, MTB, MTC, Nr, Mr>>>>>>>;

    //We can use the same matrix type for all three algorithms
    type HierA<T> = Hierarch<T, Mr, Kc, U1, Mr>;
    type HierB<T> = Hierarch<T, Kc, Nr, Nr, U1>;
    type HierC<T> = Hierarch<T, Mr, Nr, Nr, U1>;

    type HierAL3a<T> = Hierarch<T, Nr, Kc, U1, Nr>;
    type HierBL3a<T> = Hierarch<T, Kc, Mr, Mr, U1>;
    type HierCL3a<T> = Hierarch<T, Nr, Mr, U1, Nr>;

    let mut goto = <Goto<f64, HierA<f64>, HierB<f64>, HierC<f64>>>::new();
    let mut l3a = <L3A<f64, HierAL3a<f64>, HierBL3a<f64>, HierCL3a<f64>>>::new();
    let mut l3b = <L3B<f64, HierA<f64>, HierB<f64>, HierC<f64>>>::new();
    let mut l3c = <L3C<f64, HierA<f64>, HierB<f64>, HierC<f64>>>::new();

    goto.set_n_threads(4);
    l3a.set_n_threads(4);
    l3b.set_n_threads(4);
    l3c.set_n_threads(4);

    let flusher_len = 2*1024*1024; //16MB
    let mut flusher : Vec<f64> = Vec::with_capacity(flusher_len);
    for _ in 0..flusher_len {
        flusher.push(0.0);
    }

    println!("m\tn\tk\tgoto\t\tl3a\t\tl3b\t\tl3c\t\tgoto\t\tl3a\t\tl3b\t\tl3c");
    //TODO: change this to take arguments specifying problem sizes??
    for index in 01..50 {
        let size = index*64;
//        let (m, n, k) = (size, size, size);
        let m = if m_selector < 0 { size * isize::abs(m_selector) } else { m_selector };
        let n = if n_selector < 0 { size * isize::abs(n_selector) } else { n_selector };
        let k = if k_selector < 0 { size * isize::abs(k_selector) } else { k_selector };

        let n_reps = 5;
        let (goto_time, goto_err) = test_algorithm(m, n, k, &mut goto, &mut flusher, n_reps);
        let (l3a_time, l3a_err) = test_algorithm(m, n, k, &mut l3a, &mut flusher, n_reps);
        let (l3b_time, l3b_err) = test_algorithm(m, n, k, &mut l3b, &mut flusher, n_reps);
        let (l3c_time, l3c_err) = test_algorithm(m, n, k, &mut l3c, &mut flusher, n_reps);

        println!("{}\t{}\t{}\t{} \t{} \t{} \t{} \t{} \t{} \t{} \t{}", 
                 m, n, k,
                 format!("{:5.5}", util::gflops(m,n,k,goto_time)), 
                 format!("{:5.5}", util::gflops(m,n,k,l3a_time)), 
                 format!("{:5.5}", util::gflops(m,n,k,l3b_time)), 
                 format!("{:5.5}", util::gflops(m,n,k,l3c_time)), 
                 format!("{:5.5e}", goto_err.sqrt()),
                 format!("{:5.5e}", l3a_err.sqrt()),
                 format!("{:5.5e}", l3b_err.sqrt()),
                 format!("{:5.5e}", l3c_err.sqrt()));
    }

    let mut sum = 0.0;
    for a in flusher.iter() {
        sum += *a;
    }
    println!("Flush value {}", sum);
}

fn main() {
    //Best for Resident A
    test(768,-1,768);
    //Best for Resident B
    test(-1,768,768);
    //Best for Resident C
    test(768,768,-1);
}
