use std::cmp::{min, max, Ordering};

// Listen, dear Mr. Francesco Iannelli
// Your crate does such a great job
// however, I need to compare something *inside* a vector
// which means I need to specify the left pos and the right pos
// so I have to copy your floydrivest function and use it directly 

pub fn floydrivest<T, F>(a: &mut [T], nth_el: usize, mut left: usize, mut right: usize, cmp: &mut F)
where
    F: FnMut(&T, &T) -> Ordering,
    T: Clone,
{
    let mut i: usize;
    let mut j: usize;
    let mut t: T;
    while right > left {
        if right - left > 600 {
            // Use recursion on a sample of size s to get an estimate
            // for the (nth_el - left + 1 )-th smallest elementh into a[nth_el],
            // biased slightly so that the (nth_el - left + 1)-th element is expected
            // to lie in the smallest set after partitioning.
            let n: f64 = (right - left + 1) as f64;
            let i: f64 = (nth_el - left + 1) as f64;
            let z: f64 = n.ln();
            let s: f64 = 0.5 * (z * (2.0 / 3.0)).exp();
            let sn: f64 = s / n;
            let sd: f64 = 0.5 * (z * s * (1.0 - sn)).sqrt() * (i - n * 0.5).signum();

            let isn: f64 = i * s / n;
            let inner: f64 = nth_el as f64 - isn + sd;
            let ll: usize = max(left, inner as usize);
            let rr: usize = min(right, (inner + s) as usize);
            floydrivest(a, nth_el, ll, rr, cmp);
        }
        // The following code partitions a[l : r] about t, it is similar to Hoare's
        // algorithm but it'll run faster on most machines since the subscript range
        // checking on i and j has been removed.
        t = a[nth_el].clone();
        i = left;
        j = right;
        a.swap(left, nth_el);
        if cmp(&a[right], &t) == Ordering::Greater {
            a.swap(right, left);
        }
        while i < j {
            a.swap(i, j);
            i += 1;
            j -= 1;
            while cmp(&a[i], &t) == Ordering::Less {
                i += 1;
            }
            while cmp(&a[j], &t) == Ordering::Greater {
                j -= 1;
            }
        }
        if cmp(&a[left], &t) == Ordering::Equal {
            a.swap(left, j);
        } else {
            j += 1;
            a.swap(j, right);
        }
        // Now we adjust left and right so that they
        // surround the subset containing the
        // (k - left + 1)-th smallest element.
        if j <= nth_el {
            left = j + 1;
            if nth_el <= j {
                right = j.saturating_sub(1);
            }
        }
    }
}