/*
 * Copyright (c) 2023.
 *
 * This software is free software;
 *
 * You can redistribute it or modify it under terms of the MIT, Apache License or Zlib license
 */

 #![cfg(all(target_arch = "wasm32"))]
 // no way to detect if feature includes simd 
 //  #![target_feature(enable = "simd128")]
 
 //! This module provides unsafe ways to do some things
 #![allow(clippy::wildcard_imports)]
 
 use core::arch::wasm32::*;
 use core::ops::{Add, AddAssign, BitOr, BitOrAssign, Mul, MulAssign, Sub};
  
 
 /// An abstraction of an AVX ymm register that
 ///allows some things to not look ugly
 #[derive(Clone, Copy)]
 pub struct YmmRegister {
     /// An AVX register
     pub v0: v128,
     pub v1: v128,
 }
 
 impl YmmRegister {
    #[inline]
    pub unsafe fn load(src: *const i32) -> Self {
       YmmRegister{v0: *(src as *const v128), v1: *(src as *const v128).offset(1)}
    }
    
    #[inline]
    pub fn map2(self, other: Self, f: impl Fn(v128, v128) -> v128) -> Self {
        YmmRegister {
            v0:f(self.v0, other.v0),
            v1:f(self.v1, other.v1),
        }
    }

     #[inline]
     pub fn all_zero(self) -> bool {
         unsafe {
            // any value xored with itself is zero
            let zero = v128_xor(self.v0,self.v0);
            // doing the or inside instead of outside
            // because not sure how fast i32x4_all_true will be
            // but v128_or should always be quite fast
            return i32x4_all_true(i32x4_eq(v128_or(self.v0, self.v1), zero));
         }
     }
 
     #[inline]
     pub fn const_shl<const N: u32>(self) -> Self {
         // Ensure that we logically shift left
         unsafe {
            YmmRegister{
                v0: i32x4_shl(self.v0,N),
                v1: i32x4_shl(self.v1,N),
            }
         }
     }
 
     #[inline]
     pub fn const_shra<const N: u32>(self) -> Self {
         unsafe {
            YmmRegister{
                // this is the arithmetic version, logical shift is u32
                v0: i32x4_shr(self.v0,N),
                v1: i32x4_shr(self.v1,N),
            }
         }
     }
 }
 
 impl<T> Add<T> for YmmRegister
 where
     T: Into<Self>
 {
     type Output = YmmRegister;
 
     #[inline]
     fn add(self, rhs: T) -> Self::Output {
         let rhs = rhs.into();
         unsafe { self.map2(rhs, |a, b| i32x4_add(a, b)) }
     }
 }
 
 impl<T> Sub<T> for YmmRegister
 where
     T: Into<Self>
 {
     type Output = YmmRegister;
 
     #[inline]
     fn sub(self, rhs: T) -> Self::Output {
         let rhs = rhs.into();
         unsafe { self.map2(rhs, |a, b| i32x4_sub(a, b)) }
     }
 }
 
 impl<T> AddAssign<T> for YmmRegister
 where
     T: Into<Self>
 {
     #[inline]
     fn add_assign(&mut self, rhs: T) {
         let rhs: Self = rhs.into();
         *self = *self + rhs;
     }
 }
 
 impl<T> Mul<T> for YmmRegister
 where
     T: Into<Self>
 {
     type Output = YmmRegister;
 
     #[inline]
     fn mul(self, rhs: T) -> Self::Output {
         let rhs = rhs.into();
         unsafe { self.map2(rhs, |a, b| i32x4_mul(a, b)) }
     }
 }
 
 impl<T> MulAssign<T> for YmmRegister
 where
     T: Into<Self>
 {
     #[inline]
     fn mul_assign(&mut self, rhs: T) {
         let rhs: Self = rhs.into();
         *self = *self * rhs;
     }
 }
 
 impl<T> BitOr<T> for YmmRegister
 where
     T: Into<Self>
 {
     type Output = YmmRegister;
 
     #[inline]
     fn bitor(self, rhs: T) -> Self::Output {
         let rhs = rhs.into();
         unsafe { self.map2(rhs, |a, b| v128_or(a, b)) }
     }
 }
 
 impl<T> BitOrAssign<T> for YmmRegister
 where
     T: Into<Self>
 {
     #[inline]
     fn bitor_assign(&mut self, rhs: T) {
         let rhs: Self = rhs.into();
         *self = *self | rhs;
     }
 }
 
 impl From<i32> for YmmRegister {
     #[inline]
     fn from(val: i32) -> Self {
         unsafe {
             let dup = i32x4_splat(val);
 
             YmmRegister {
                 v0: dup,
                 v1: dup,
             }
         }
     }
 }
 
 #[allow(clippy::too_many_arguments)]
 #[inline]
 unsafe fn transpose4(
     v0: &mut v128, v1: &mut v128, v2: &mut v128, v3: &mut v128
 ) {
    /* don't trust was shuffle's instruction performance,
    * (no neon implementation possible, etc)
    * so doing this with 64 bit operations as a workaround
    * which should be much more consistently performant, 
    * even if more instructions generated
    */
    // flips the 64 bit blocks around with lane extraction (most reliable option I could find)
    let w0 = i64x2_replace_lane::<1>(*v0, i64x2_extract_lane::<0>(*v2));
    let w1 = i64x2_replace_lane::<1>(*v1, i64x2_extract_lane::<0>(*v3));
    let w2 = i64x2_replace_lane::<0>(*v2, i64x2_extract_lane::<1>(*v0));
    let w3 = i64x2_replace_lane::<0>(*v3, i64x2_extract_lane::<1>(*v1));
    // now use bit manipulations to do vertical swapping of the 32 bit blocks inside the 64 bit blocks
    let low_mask = i64x2_splat(0xffffffff);
    let high_mask = v128_not(low_mask);
    // note that u64_shr is for logical shift, which is what we want here, because
    // we are using shifting to swap around the 32 bit elements in the 64 bit vectors
    *v0 = v128_or(v128_and(w0, low_mask),i64x2_shl(w1, 32));
    *v1 = v128_or(u64x2_shr(w1, 32),v128_and(w0, high_mask));
    *v2 = v128_or(v128_and(w2, low_mask),i64x2_shl(w3, 32));
    *v3 = v128_or(u64x2_shr(w3, 32),v128_and(w2, high_mask));
 }
 
 /// Transpose an array of 8 by 8 i32
 /// Arm has dedicated interleave/transpose instructions
 /// we:
 /// 1. Transpose the upper left and lower right quadrants
 /// 2. Swap and transpose the upper right and lower left quadrants
 #[allow(clippy::too_many_arguments)]
 #[inline]
 pub unsafe fn transpose(
     v0: &mut YmmRegister, v1: &mut YmmRegister, v2: &mut YmmRegister, v3: &mut YmmRegister,
     v4: &mut YmmRegister, v5: &mut YmmRegister, v6: &mut YmmRegister, v7: &mut YmmRegister
 ) {
     use core::mem::swap;
 
     let ul0 = &mut v0.v0;
     let ul1 = &mut v1.v0;
     let ul2 = &mut v2.v0;
     let ul3 = &mut v3.v0;
 
     let ur0 = &mut v0.v1;
     let ur1 = &mut v1.v1;
     let ur2 = &mut v2.v1;
     let ur3 = &mut v3.v1;
 
     let ll0 = &mut v4.v0;
     let ll1 = &mut v5.v0;
     let ll2 = &mut v6.v0;
     let ll3 = &mut v7.v0;
 
     let lr0 = &mut v4.v1;
     let lr1 = &mut v5.v1;
     let lr2 = &mut v6.v1;
     let lr3 = &mut v7.v1;
 
     swap(ur0, ll0);
     swap(ur1, ll1);
     swap(ur2, ll2);
     swap(ur3, ll3);
 
     transpose4(ul0, ul1, ul2, ul3);
 
     transpose4(ur0, ur1, ur2, ur3);
 
     transpose4(ll0, ll1, ll2, ll3);
 
     transpose4(lr0, lr1, lr2, lr3);
 }
 
 #[cfg(test)]
 mod tests {
     use super::*;
 
     #[test]
     fn test_transpose() {
         fn get_val(i: usize, j: usize) -> i32 {
             ((i * 8) / (j + 1)) as i32
         }
         unsafe {
             let mut vals: [i32; 8 * 8] = [0; 8 * 8];
 
             for i in 0..8 {
                 for j in 0..8 {
                     // some order-dependent value of i and j
                     let value = get_val(i, j);
                     vals[i * 8 + j] = value;
                 }
             }
 
             let mut regs: [YmmRegister; 8] = core::mem::transmute(vals);
             let mut reg0 = regs[0];
             let mut reg1 = regs[1];
             let mut reg2 = regs[2];
             let mut reg3 = regs[3];
             let mut reg4 = regs[4];
             let mut reg5 = regs[5];
             let mut reg6 = regs[6];
             let mut reg7 = regs[7];
 
             transpose(
                 &mut reg0, &mut reg1, &mut reg2, &mut reg3, &mut reg4, &mut reg5, &mut reg6,
                 &mut reg7
             );
 
             regs[0] = reg0;
             regs[1] = reg1;
             regs[2] = reg2;
             regs[3] = reg3;
             regs[4] = reg4;
             regs[5] = reg5;
             regs[6] = reg6;
             regs[7] = reg7;
 
             let vals_from_reg: [i32; 8 * 8] = core::mem::transmute(regs);
 
             for i in 0..8 {
                 for j in 0..i {
                     let orig = vals[i * 8 + j];
                     vals[i * 8 + j] = vals[j * 8 + i];
                     vals[j * 8 + i] = orig;
                 }
             }
 
             for i in 0..8 {
                 for j in 0..8 {
                     assert_eq!(vals[j * 8 + i], get_val(i, j));
                     assert_eq!(vals_from_reg[j * 8 + i], get_val(i, j));
                 }
             }
 
             assert_eq!(vals, vals_from_reg);
         }
     }
 }
 