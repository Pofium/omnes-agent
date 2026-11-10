//! SIMD-ядра векторной математики (Ф6.3 ob2h v2.0 / Ф72b).
//!
//! `dot`/`norm_sq`/`cosine` по умолчанию выполняются на AVX2+FMA (x86_64) с
//! рантайм-детекцией через `is_x86_feature_detected!`; на прочих платформах и
//! при отсутствии AVX2 используется скалярный fallback. Результаты совпадают с
//! прежней скалярной реализацией в пределах f32-погрешности (точные значения
//! для единичных/нулевых векторов сохраняются бит-в-бит).

use std::sync::OnceLock;

/// Указатель на ядро скалярного произведения.
type DotKernel = unsafe fn(&[f32], &[f32]) -> f32;

/// Скалярный fallback: та же семантика, что у исторической `cosine`.
fn dot_scalar(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());
    let mut acc = 0.0f32;
    for i in 0..n {
        acc += a[i] * b[i];
    }
    acc
}

/// AVX2+FMA ядро: 8 f32 за итерацию, горизонтальная сумма, «хвост» скалярно.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2", enable = "fma")]
unsafe fn dot_avx2(a: &[f32], b: &[f32]) -> f32 {
    use std::arch::x86_64::*;

    let n = a.len().min(b.len());
    // SAFETY: `i + 8 <= n` гарантирует, что обе загрузки в границах срезов;
    // «хвост» читается через `get_unchecked` при `i < n == min(длины)`.
    unsafe {
        let mut acc = _mm256_setzero_ps();
        let mut i = 0usize;
        while i + 8 <= n {
            let va = _mm256_loadu_ps(a.as_ptr().add(i));
            let vb = _mm256_loadu_ps(b.as_ptr().add(i));
            acc = _mm256_fmadd_ps(va, vb, acc);
            i += 8;
        }

        let hi = _mm256_extractf128_ps(acc, 1);
        let lo = _mm256_castps256_ps128(acc);
        let sum128 = _mm_add_ps(hi, lo);
        let shuf = _mm_movehdup_ps(sum128);
        let sums = _mm_add_ps(sum128, shuf);
        let shuf2 = _mm_movehl_ps(sums, sums);
        let sums2 = _mm_add_ss(sums, shuf2);
        let mut total = _mm_cvtss_f32(sums2);

        while i < n {
            total += *a.get_unchecked(i) * *b.get_unchecked(i);
            i += 1;
        }
        total
    }
}

/// Однократный выбор ядра по возможностям CPU (кэшируется в `OnceLock`).
fn dot_kernel() -> DotKernel {
    static KERNEL: OnceLock<DotKernel> = OnceLock::new();
    *KERNEL.get_or_init(|| {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
                return dot_avx2;
            }
        }
        dot_scalar
    })
}

/// Скалярное произведение (SIMD при поддержке CPU). Длины усекаются до min.
pub fn dot(a: &[f32], b: &[f32]) -> f32 {
    // SAFETY: ядро выбирается из двух известных корректных функций; длины
    // усекаются до min внутри ядра.
    unsafe { (dot_kernel())(a, b) }
}

/// Квадрат L2-нормы (через то же ядро).
pub fn norm_sq(a: &[f32]) -> f32 {
    dot(a, a)
}

/// Косинусное сходство. Совместимо с исторической скалярной семантикой:
/// при несовпадении длин, пустом входе или нулевой норме возвращает `0.0`.
pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot = dot(a, b);
    let denom = norm_sq(a).sqrt() * norm_sq(b).sqrt();
    if denom == 0.0 { 0.0 } else { dot / denom }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Эталонная скалярная реализация (независимая от ядра).
    fn scalar_cosine(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        let mut dot = 0.0f32;
        let mut na = 0.0f32;
        let mut nb = 0.0f32;
        for (x, y) in a.iter().zip(b.iter()) {
            dot += x * y;
            na += x * x;
            nb += y * y;
        }
        let denom = na.sqrt() * nb.sqrt();
        if denom == 0.0 { 0.0 } else { dot / denom }
    }

    fn sample(len: usize, seed: u64) -> Vec<f32> {
        // Детерминированный псевдослучайный вектор без внешних зависимостей.
        let mut state = seed | 1;
        (0..len)
            .map(|_| {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                ((state >> 33) as f32 / u32::MAX as f32) * 2.0 - 1.0
            })
            .collect()
    }

    #[test]
    fn dot_matches_scalar_including_tail() {
        for len in [1usize, 3, 7, 8, 9, 16, 31, 384, 385] {
            let a = sample(len, 42);
            let b = sample(len, 7);
            let simd = dot(&a, &b);
            let scalar = dot_scalar(&a, &b);
            assert!(
                (simd - scalar).abs() < 1e-3,
                "len={len}: simd={simd} scalar={scalar}"
            );
        }
    }

    #[test]
    fn cosine_matches_scalar_reference() {
        for len in [1usize, 2, 8, 15, 384] {
            let a = sample(len, 11);
            let b = sample(len, 99);
            let got = cosine(&a, &b);
            let want = scalar_cosine(&a, &b);
            assert!(
                (got - want).abs() < 1e-4,
                "len={len}: got={got} want={want}"
            );
        }
    }

    #[test]
    fn cosine_exact_for_unit_and_zero_vectors() {
        assert!((cosine(&[1.0, 0.0, 0.0], &[1.0, 0.0, 0.0]) - 1.0).abs() < 1e-6);
        assert!((cosine(&[1.0, 0.0], &[-1.0, 0.0]) + 1.0).abs() < 1e-6);
        assert_eq!(cosine(&[0.0, 0.0], &[1.0, 1.0]), 0.0);
    }

    #[test]
    fn cosine_guards_mismatch_and_empty() {
        assert_eq!(cosine(&[1.0], &[1.0, 2.0]), 0.0);
        assert_eq!(cosine(&[], &[]), 0.0);
        assert_eq!(norm_sq(&[]), 0.0);
    }
}
