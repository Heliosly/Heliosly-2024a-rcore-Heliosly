//! Implementation of physical and virtual address and page number.
use super::PageTableEntry;
use crate::
    config::{
        PAGE_SIZE,
        
        KERNEL_DIRECT_OFFSET, PAGE_SIZE_BITS,
    };
use core::fmt::{self, Debug, Formatter};
/// physical address
const PA_WIDTH_SV39: usize = 56;
///
pub const VA_WIDTH_SV39: usize = 39;
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BITS;
const VPN_WIDTH_SV39: usize = VA_WIDTH_SV39 - PAGE_SIZE_BITS;

/// kernel address
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct KernelAddr(pub usize);

/// physical address
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub usize);
/// virtual address
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub usize);
/// physical page number
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPageNum(pub usize);
/// virtual page number
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPageNum(pub usize);

/// Debugging

impl Debug for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VA:{:#x}", self.0))
    }
}
impl Debug for VirtPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VPN:{:#x}", self.0))
    }
}
impl Debug for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PA:{:#x}", self.0))
    }
}
impl Debug for PhysPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PPN:{:#x}", self.0))
    }
}

/// T: {PhysAddr, VirtAddr, PhysPageNum, VirtPageNum}
/// T -> usize: T.0
/// usize -> T: usize.into()

impl From<usize> for KernelAddr {
    fn from(v: usize) -> Self {
        Self(v)
    }
}
impl From<PhysAddr> for KernelAddr {
    fn from(pa: PhysAddr) -> Self {
        Self(pa.0 + (KERNEL_DIRECT_OFFSET << PAGE_SIZE_BITS))
    }
}

impl From<KernelAddr> for PhysAddr {
    fn from(ka: KernelAddr) -> Self {
        Self(ka.0 - (KERNEL_DIRECT_OFFSET << PAGE_SIZE_BITS))
    }
}

impl From<KernelAddr> for VirtAddr {
    fn from(ka: KernelAddr) -> Self {
        Self(ka.0)
    }
}

impl From<usize> for PhysAddr {
    fn from(v: usize) -> Self {
        // Self(v & ((1 << PA_WIDTH_SV39) - 1))
        let tmp = v as isize >> PA_WIDTH_SV39;
        assert!(tmp == 0 || tmp == -1);
        Self(v)
    }
}
impl From<usize> for PhysPageNum {
    fn from(v: usize) -> Self {
        // Self(v & ((1 << PPN_WIDTH_SV39) - 1))
        let tmp = (v as isize >> PPN_WIDTH_SV39) as isize;
        //trace!("tmp:{:x},v:{:x}",tmp,v);
        assert!(tmp == 0 || tmp == -1);
        Self(v)
    }
}
impl From<KernelAddr> for PhysPageNum {
    fn from(ka: KernelAddr) -> Self {
        let pa = PhysAddr::from(ka);
        pa.floor()
    }
}

// impl TryFrom<usize> for VirtAddr {
//     fn try_from(v: usize) -> Result<Self, Self::Error> {
//         let tmp = (v   >> VA_WIDTH_SV39)  ;
//         if tmp != 0 && tmp != -1 {
//             log::error!("v {:#x}, tmp {:#x}", v, tmp);
//             local_hart().env().stack_tracker.print_stacks_err();
//             return Err(SyscallErr::EFAULT);
//         }
//         Ok(Self(v))
//     }
// }

impl From<usize> for VirtAddr {
    fn from(v: usize) -> Self {
        // Self(v & ((1 << VA_WIDTH_SV39) - 1))
        let tmp = (v as isize >> VA_WIDTH_SV39) as isize;
        if tmp != 0 && tmp != -1 {
            #[allow(clippy::empty_loop)]
            loop{

            }
            //log::error!("v {:#x}, tmp {:#x}", v, tmp);
          
        }
        assert!(tmp == 0 || tmp == -1, "invalid va: {:#x}", v);
        Self(v)
    }
}
impl From<usize> for VirtPageNum {
    fn from(v: usize) -> Self {
        // Self(v & ((1 << VPN_WIDTH_SV39) - 1))
        let tmp = v >> (VPN_WIDTH_SV39 - 1);
        assert!(tmp == 0 || tmp == (1 << (52 - VPN_WIDTH_SV39 + 1)) - 1);
        // let tmp = ((v  ) >> VPN_WIDTH_SV39)  ;
        // if tmp != 0 && tmp != -1 {
        //     error!("tmp {:#x}, v {:#x}", tmp, v);
        // }
        // assert!(tmp == 0 || tmp == -1);
        Self(v)
    }
}
impl From<PhysAddr> for usize {
    fn from(v: PhysAddr) -> Self {
        v.0
    }
}
impl From<PhysPageNum> for usize {
    fn from(v: PhysPageNum) -> Self {
        v.0
    }
}
impl From<VirtAddr> for usize {
    fn from(v: VirtAddr) -> Self {
        if v.0 >= (1 << (VA_WIDTH_SV39 - 1)) {
            v.0 | (!((1 << VA_WIDTH_SV39) - 1))
        } else {
            v.0
        }
    }
}
impl From<VirtPageNum> for usize {
    fn from(v: VirtPageNum) -> Self {
        v.0
    }
}
///
impl VirtAddr {
    ///`VirtAddr`->`VirtPageNum`
    pub fn floor(&self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }
    ///`VirtAddr`->`VirtPageNum`
    pub fn ceil(&self) -> VirtPageNum {
        VirtPageNum((self.0 - 1 + PAGE_SIZE) / PAGE_SIZE)
    }
    ///Get page offset
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
    ///Check page aligned
    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
}
impl From<VirtAddr> for VirtPageNum {
    fn from(v: VirtAddr) -> Self {
        assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}
impl From<VirtPageNum> for VirtAddr {
    fn from(v: VirtPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}
impl PhysAddr {
    ///Get mutable reference to `PhysAddr` value
    pub fn get_ref<T>(&self) -> &'static T {
        unsafe { (self.0 as *mut T).as_ref().unwrap() }
    }
    ///Get mutable reference to `PhysAddr` value
    pub fn get_mut<T>(&self) -> &'static mut T {
        unsafe { (self.0 as *mut T).as_mut().unwrap() }
    }
    ///`PhysAddr`->`PhysPageNum`
    pub fn floor(&self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }
    ///`PhysAddr`->`PhysPageNum`
    pub fn ceil(&self) -> PhysPageNum {
        PhysPageNum((self.0 - 1 + PAGE_SIZE) / PAGE_SIZE)
    }
    ///Get page offset
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
    ///Check page aligned
    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
}
impl From<PhysAddr> for PhysPageNum {
    fn from(v: PhysAddr) -> Self {
        assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}
impl From<PhysPageNum> for PhysAddr {
    fn from(v: PhysPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}

impl VirtPageNum {
    ///Return VPN 3 level indices
    pub fn indexes(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut indices = [0usize; 3];
        for i in (0..3).rev() {
            indices[i] = vpn & 511;
            vpn >>= 9;
        }
        indices
    }
     /// Get `PageTableEntry` on `PhysPageNum`
     pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        let va: VirtAddr = VirtAddr::from(*self);
   
        unsafe { core::slice::from_raw_parts_mut(va.0 as *mut PageTableEntry, 512) }
    }
    ///d
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let va: VirtAddr = VirtAddr::from(*self);
        
        
        unsafe { core::slice::from_raw_parts_mut(va.0 as *mut u8, 4096) }
    }

}

impl KernelAddr {
    ///Get mutable reference to `PhysAddr` value
    pub fn get_ref<T>(&self) -> &'static T {
        unsafe { (self.0 as *mut T).as_ref().unwrap() }
    }
    ///Get mutable reference to `PhysAddr` value
    pub fn get_mut<T>(&self) -> &'static mut T {
        unsafe { (self.0 as *mut T).as_mut().unwrap() }
    }
}
impl PhysPageNum {
    /// Get `PageTableEntry` on `PhysPageNum`
    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        let pa: PhysAddr = (*self).into();
        let kernel_pa = KernelAddr::from(pa).0;
        unsafe { core::slice::from_raw_parts_mut(kernel_pa as *mut PageTableEntry, 512) }
    }
    ///d
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: PhysAddr = (*self).into();
        let kernel_pa = KernelAddr::from(pa).0;
        
        unsafe { core::slice::from_raw_parts_mut(kernel_pa as *mut u8, 4096) }
    }

    ///d
    pub fn get_ref<T>(&self) -> &'static T {
        let pa: PhysAddr = (*self).into();
        let kernel_pa = KernelAddr::from(pa);
        kernel_pa.get_ref()
    }

    ///d
    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa: PhysAddr = (*self).into();
        let kernel_pa = KernelAddr::from(pa);
        kernel_pa.get_mut()
    }
}

/// step the give type
pub trait StepByOne {
    /// Step the give type
    fn step(&mut self);
}

impl StepByOne for VirtAddr {
    fn step(&mut self) {
        self.0 += 1;
    }
}

impl StepByOne for VirtPageNum {
    fn step(&mut self) {
        self.0 += 1;
    }
}

impl StepByOne for PhysPageNum {
    fn step(&mut self) {
        self.0 += 1;
    }
}

#[derive(Copy, Clone, Debug)]
/// a simple range structure for type T
pub struct SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    l: T,
    r: T,
}

impl<T> SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    pub fn new(start: T, end: T) -> Self {
        assert!(start <= end, "start {:?} > end {:?}!", start, end);
        Self { l: start, r: end }
    }
    pub fn get_start(&self) -> T {
        self.l
    }
    pub fn get_end(&self) -> T {
        self.r
    }
    /// Note that the new right bound cannot be smaller than left bound
    pub fn modify_right_bound(&mut self, new_right: T) {
        assert!(new_right >= self.l);
        self.r = new_right;
    }
}

impl<T> IntoIterator for SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    type Item = T;
    type IntoIter = SimpleRangeIterator<T>;
    fn into_iter(self) -> Self::IntoIter {
        SimpleRangeIterator::new(self.l, self.r)
    }
}

// impl<T> Iterator for SimpleRange<T>
// where
//     T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
// {

// }

/// iterator for the simple range structure
pub struct SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    current: T,
    end: T,
}
impl<T> SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    pub fn new(l: T, r: T) -> Self {
        Self { current: l, end: r }
    }
}
impl<T> Iterator for SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end {
            None
        } else {
            let t = self.current;
            self.current.step();
            Some(t)
        }
    }
}
/// a simple range structure for virtual page number
pub type VPNRange = SimpleRange<VirtPageNum>;
