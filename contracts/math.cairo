from starkware.cairo.common.cairo_builtins import HashBuiltin
from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.math import abs_value, signed_div_rem, sqrt
from starkware.cairo.common.math_cmp import is_le, is_nn

from contracts.constants import SCALE_FP, SCALE_FP_SQRT, RANGE_CHECK_BOUND, PI_fp, HALF_PI_fp

// Takes square root of fixed point quantity "x_fp"
func sqrt_fp{range_check_ptr}(x_fp: felt) -> felt {
    let x_ = sqrt(x_fp);  // notice: sqrt() now returns a single felt, not a tuple anymore (tuple is returned for cairo < 0.10)
    let y_fp = x_ * SCALE_FP_SQRT;  // compensate for the square root
    return y_fp;
}

// Multiplies fixed point quantity "a_fp" by fixed point quantity "b_fp", with range check
func mul_fp{range_check_ptr}(a_fp: felt, b_fp: felt) -> felt {
    // signed_div_rem by SCALE_FP after multiplication
    tempvar product = a_fp * b_fp;
    let (c_fp, _) = signed_div_rem(product, SCALE_FP, RANGE_CHECK_BOUND);
    return c_fp;
}

// Divides fixed point quantity "a_fp" by fixed point quantity "b_fp", with range check
func div_fp{range_check_ptr}(a_fp: felt, b_fp: felt) -> felt {
    // multiply by SCALE_FP before signed_div_rem
    tempvar a_scaled = a_fp * SCALE_FP;
    // 2nd argument in signed_div_rem must be positive
    let bool = is_nn(b_fp);
    if (bool == 0) {
        // if b_fp < 0, then use negatives of first two arguments
        let (c_fp, _) = signed_div_rem(-a_scaled, -b_fp, RANGE_CHECK_BOUND);
        return c_fp;
    }
    let (c_fp, _) = signed_div_rem(a_scaled, b_fp, RANGE_CHECK_BOUND);
    return c_fp;
}

// Multiplies fixed point quantity "a_fp" by non-fixed point quantity "b"
func mul_fp_nfp{range_check_ptr}(a_fp: felt, b: felt) -> felt {
    let c_fp = a_fp * b;
    return c_fp;
}

// Divides fixed point quantity "a_fp" by non-fixed point quantity "b", with range check
func div_fp_nfp{range_check_ptr}(a_fp: felt, b: felt) -> felt {
    // 2nd argument in signed_div_rem must be positive
    let bool = is_nn(b);
    if (bool == 0) {
        // if b_fp < 0, then use negatives of first two arguments
        let (c_fp, _) = signed_div_rem(-a_fp, -b, RANGE_CHECK_BOUND);
        return c_fp;
    }
    let (c_fp, _) = signed_div_rem(a_fp, b, RANGE_CHECK_BOUND);
    return c_fp;
}

// Finds distance between fixed point coordinate values (x_1, y_1) and (x_2, y_2)
func distance_two_points_fp{range_check_ptr}(
    x_1_fp: felt, y_1_fp: felt, x_2_fp: felt, y_2_fp: felt
) -> felt {
    let x_diff_fp = x_2_fp - x_1_fp;
    let y_diff_fp = y_2_fp - y_1_fp;
    let x_diff_sq_fp = mul_fp(x_diff_fp, x_diff_fp);
    let y_diff_sq_fp = mul_fp(y_diff_fp, y_diff_fp);
    let sum_fp = x_diff_sq_fp + y_diff_sq_fp;
    let r_fp = sqrt_fp(sum_fp);
    return r_fp;
}

// //////////////////////////////////////////////////////
// Below are two versions of Taylor series approximation for cosine:
//   - Using either 5 (8th order) or 4 (6th order) terms
//   - May be able to use only 4 terms if -pi/2 <= theta <= pi/2
// //////////////////////////////////////////////////////

// Taylor series approximation of cosine(theta) for FP theta value
// Uses 5 terms (to 8th order)
// NO SHIFT: Assumes -pi <= theta <= +pi
func cosine_8th_fp{range_check_ptr}(theta_fp: felt) -> (value_fp: felt) {
    //
    // cos(theta) ~= 1 - theta^2/2! + theta^4/4! - theta^6/6! + theta^8/8!

    let theta_2_fp = mul_fp(theta_fp, theta_fp);
    let theta_4_fp = mul_fp(theta_2_fp, theta_2_fp);
    let theta_6_fp = mul_fp(theta_2_fp, theta_4_fp);
    let theta_8_fp = mul_fp(theta_2_fp, theta_6_fp);

    let theta_2_div2_fp = div_fp_nfp(theta_2_fp, 2);
    let theta_4_div24_fp = div_fp_nfp(theta_4_fp, 24);
    let theta_6_div720_fp = div_fp_nfp(theta_6_fp, 720);
    let theta_8_div40320_fp = div_fp_nfp(theta_8_fp, 40320);

    let value_fp = 1 * SCALE_FP - theta_2_div2_fp + theta_4_div24_fp - theta_6_div720_fp + theta_8_div40320_fp;

    return (value_fp=value_fp);
}

// Taylor series approximation of cosine(theta) for FP theta value
// Uses 4 terms (to 6th order)
// NO SHIFT: Assumes -pi <= theta <= +pi, but is best used with -pi/2 <= theta <= pi/2
func cosine_6th_fp{range_check_ptr}(theta_fp: felt) -> (value_fp: felt) {
    // cos(theta) ~= 1 - theta^2/2! + theta^4/4! - theta^6/6!
    let theta_2_fp = mul_fp(theta_fp, theta_fp);
    let theta_4_fp = mul_fp(theta_2_fp, theta_2_fp);
    let theta_6_fp = mul_fp(theta_2_fp, theta_4_fp);

    let theta_2_div2_fp = div_fp_nfp(theta_2_fp, 2);
    let theta_4_div24_fp = div_fp_nfp(theta_4_fp, 24);
    let theta_6_div720_fp = div_fp_nfp(theta_6_fp, 720);

    let value_fp = 1 * SCALE_FP - theta_2_div2_fp + theta_4_div24_fp - theta_6_div720_fp;

    return (value_fp=value_fp);
}

// Cosine approximation:
//   Taylor series approximation is more accurate if -pi/2 <= theta_0 <= +pi/2. So:
//   If theta_0 is in 2nd/3rd quadrant:
//     (1) move angle to 1st/4th quadrant for cosine approximation
//     (2) force negative sign for cosine(theta_0)
//   (Use theta_0_deg for comparisons because calculated theta_0 in radians is slightly rounded)
//   Then call cosine_6th_fp or cosine_8th_fp
func cosine_approx{range_check_ptr}(theta_0_fp: felt, theta_0_deg: felt) -> (cos_theta_0_fp: felt) {
    alloc_locals;
    local range_check_ptr = range_check_ptr;

    let bool1 = is_le(90, theta_0_deg);
    if (bool1 == 1) {
        if (theta_0_deg == 90) {
            // If 90 degrees, use exact value of cos_theta_0
            tempvar cos_theta_0_fp = 0;
            tempvar range_check_ptr = range_check_ptr;
        } else {
            // If in 2nd quadrant, move to 1st, but force cos_theta_0_fp to be negative:
            let theta_0_moved_fp = PI_fp - theta_0_fp;
            // let (non_neg_cos_theta_0_fp) = cosine_6th_fp(theta_0_moved_fp);
            let (non_neg_cos_theta_0_fp) = cosine_8th_fp(theta_0_moved_fp);
            tempvar cos_theta_0_fp = -non_neg_cos_theta_0_fp;
            tempvar range_check_ptr = range_check_ptr;
        }
    } else {
        let bool2 = is_le(theta_0_deg, -90);
        if (bool2 == 1) {
            if (theta_0_deg == -90) {
                // If -90 degrees, use exact value of cos_theta_0
                tempvar cos_theta_0_fp = 0;
                tempvar range_check_ptr = range_check_ptr;
            } else {
                // If in 3rd quadrant, move to 4th, but force cos_theta_0_fp to be negative:
                let theta_0_moved_fp = (-PI_fp) - theta_0_fp;
                // let (non_neg_cos_theta_0_fp) = cosine_6th_fp(theta_0_moved_fp);
                let (non_neg_cos_theta_0_fp) = cosine_8th_fp(theta_0_moved_fp);
                tempvar cos_theta_0_fp = -non_neg_cos_theta_0_fp;
                tempvar range_check_ptr = range_check_ptr;
            }
        } else {
            // If in 1st or 4th quadrant, all is good
            // let (temp_cos_theta_0_fp) = cosine_6th_fp(theta_0_fp);
            let (temp_cos_theta_0_fp) = cosine_8th_fp(theta_0_fp);

            tempvar cos_theta_0_fp = temp_cos_theta_0_fp;
            tempvar range_check_ptr = range_check_ptr;
        }
    }

    return (cos_theta_0_fp=cos_theta_0_fp);
}

// Sine approximation: need to force correct signs
func sine_approx{range_check_ptr}(theta_0_fp: felt, cos_theta_0_fp: felt) -> (
    sin_theta_0_fp: felt
) {
    alloc_locals;
    local range_check_ptr = range_check_ptr;

    let cos_theta_0_squared_fp = mul_fp(cos_theta_0_fp, cos_theta_0_fp);
    let diff_fp = 1 * SCALE_FP - cos_theta_0_squared_fp;
    let root = sqrt_fp(diff_fp);

    let bool = is_nn(theta_0_fp);
    if (bool == 1) {
        // If theta_0 >= 0, then sin_theta_0 >= 0
        tempvar sin_theta_0_fp = root;
        tempvar range_check_ptr = range_check_ptr;
    } else {
        // If theta_0 < 0, then sin_theta_0 < 0
        tempvar sin_theta_0_fp = -root;
        tempvar range_check_ptr = range_check_ptr;
    }

    return (sin_theta_0_fp=sin_theta_0_fp);
}
