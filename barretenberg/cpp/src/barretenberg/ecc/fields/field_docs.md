Prime field documentation    {#field_docs}
===
Barretenberg has its own implementation of finite field arithmetic. The implementation targets 254 (bn254, grumpkin) and 256-bit (secp256k1, secp256r1) fields. Internally the field is representate as a little-endian C-array of 4 uint64_t limbs.

## Field arithmetic
### Introduction to Montgomery form {#field_docs_montgomery_explainer}
We use Montgomery reduction to speed up field multiplication. For an original element  \f$ a ∈ F_p\f$ the element is represented internally as $$ a⋅R\ mod\ p$$ where \f$R = 2^d\ mod\ p\f$. The chosen \f$d\f$ depends on the build configuration:
1. \f$d=29⋅9=261\f$ for builds that don't support the uint128_t type, for example, for WASM build
2. \f$d=64⋅4=256\f$ for standard builds (x86_64).

The goal of using Montgomery form is to avoid heavy division modulo \f$p\f$. To compute a representative of element $$c = a⋅b\ mod\ p$$ we compute $$c⋅R = (a⋅R)⋅(b⋅R) / R\ mod\ p$$, but we use an efficient division trick to avoid straight modular division. Let's look into the standard 4⋅64 case:
1. First, we compute the value $$c_r=c⋅R⋅R = aR⋅bR$$ in integers and get a value with 8 64-bit limbs
2. Then we take the lowest limb of \f$c_r\f$ (\f$c_r[0]\f$) and multiply it by a special value $$r_{inv} = -1 ⋅ p^{-1}\ mod\  2^{64}$$ As a result we get $$k = r_{inv}⋅ c_r[0]\ mod\ 2^{64}$$
3. Next we update \f$c_r\f$ in integers by adding a value \f$k⋅p\f$: $$c_r += k⋅p$$ You might notice that the value of \f$c_r\ mod\ p\f$ hasn't changed, since we've added a multiple of the modulus. A the same time, if we look at the expression modulo \f$2^{64}\f$: $$c_r + k⋅p = c_r + c_r⋅r_{inv}⋅p = c_r + c_r⋅ (-1)⋅p^{-1}⋅p = c_r - c_r = 0\ mod\ 2^{64}$$ The result is equivalent modulo \f$p\f$, but we zeroed out the lowest limb
4. We perform the same operation for \f$c_r[1]\f$, but instead of adding \f$k⋅p\f$, we add \f$2^{64}⋅k⋅p\f$. In the implementation, instead of adding \f$k⋅ p\f$ to limbs of \f$c_r\f$ starting with zero, we just start with limb 1. This ensures that \f$c_r[1]=0\f$. We then perform the same operation for 2 more limbs.
5. At this stage we are left with a version of \f$c_r\f$ where the first 4 limbs of the total 8 limbs are zero. So if we treat the 4 high limbs as a separate integer \f$c_{r.high}\f$, $$c_r = c_{r.high}⋅2^{256}=c_{r.high}⋅R\ mod\ p \Rightarrow c_{r.high} = c\cdot R\ mod\ p$$ and we can get the evaluation simply by taking the 4 high limbs of \f$c_r\f$.
6. The previous step has reduced the intermediate value of \f$cR\f$ to range \f$[0,2p)\f$, so we must check if it is more than \f$p\f$ and subtract the modulus once if it overflows.

Why does this work? Originally both \f$aR\f$ and \f$bR\f$ are less than the modulus \f$p\f$ in integers, so $$aR\cdot bR <= (p-1)^2$$ During each of the \f$k\cdot p\f$ addition rounds we can add at most \f$(2^{64}-1)p\f$ to corresponding digits, so at most we add \f$(2^{256}-1)p\f$ and the total is $$aR\cdot bR + k_{0,1,2,3}p \le (p-1)^2+(2^{256}-1)p < 2\cdot 2^{256}p \Rightarrow c_{r.high} = \frac{aR\cdot bR + k_{0,1,2,3}p}{2^{256}} < 2p$$.

For bn254 scalar and base fields we can do even better by employing a simple trick. The moduli of both fields are 254 bits, while 4 64-bit limbs allow 256 bits of storage. We relax the internal representation to use values in range \f$[0,2p)\f$. The addition, negation and subtraction operation logic doesn't change, we simply replace the modulus \f$p\f$ with \f$2p\f$, but the mutliplication becomes more efficient. The multiplicands are in range \f$[0,2p)\f$, but we add multiples of modulus \f$p\f$ to reduce limbs, not \f$2p\f$. If we revisit the \f$c_r\f$ formula:
$$aR\cdot bR + k_{0,1,2,3}p \le (2p-1)^2+(2^{256}-1)p = 2^{256}p+4p^2-5p+1 \Rightarrow$$ $$\Rightarrow c_{r.high} = \frac{aR\cdot bR + k_{0,1,2,3}p}{2^{256}} \le \frac{2^{256}p+4p^2-5p+1}{2^{256}}=p +\frac{4p^2 - 5p +1}{2^{256}}, 4p < 2^{256} \Rightarrow$$ $$\Rightarrow p +\frac{4p^2 - 5p +1}{2^{256}} < 2p$$ So we ended in the same range and we don't have to perform additional reductions.

**N.B.** In the code we refer to this form as coarse




### Converting to and from Montgomery form
Obviously we want to avoid using standard form division when converting between forms, so we use Montgomery form to convert to Montgomery form. If we look at a value \f$a\ mod\ p\f$ we can notice that this is the Montgomery form of \f$a\cdot R^{-1}\ mod\ p\f$, so if we want to get \f$aR\f$ from it, we need to multiply it by the Montgomery form of \f$R\ mod\ p\f$, which is \f$R\cdot R\ mod\ p\f$. So using Montgomery multiplication we compute

$$a \cdot R^2 / R  = a\cdot R\ mod\ p$$

To convert from Montgomery form into standard form we multiply the element in Montgomery form by 1:

$$ aR \cdot 1 / R = a\ mod\ p$$

## Architecture details {#field_docs_architecture_details}
You could say that for each multiplication or squaring primitive there are 3 implementations:
1. Generic 64-bit implementation when uint128_t type is available (there is efficient multiplication of 64-bit values)
2. Assembly 64-bit implementation (Intel ADX and no Intel ADX versions)
3. Implementation targeting WASM

The generic implementation has 2 purposes:
1. Building barretenberg on platforms we haven't targetted in the past (new ARM-based Macs, for example)
2. Compile-time computation of constant expressions, since we can't use the assembly implementation for those.

The assembly implementation for x86_64 is optimised. There are 2 versions:
1. General x86_64 implementation that uses 64-bit registers. The squaring operation is equivalent to multiplication for simplicity and because the original squaring implementation was quite buggy.
2. Implementation using Intel ADX. It allows simultaneous use of two addition-with carry operations (adox and adcx) on two separate CPU gates (units of execution that can work simultaneously on the same core), which almost halves the time spent adding up the results of uint64_t multiplication.

Implementation for WASM:

We use 9 29-bit limbs for computation (storage stays the same) and we change the Montgomery form. The reason for a different architecture is that WASM doesn't have:
1. 128-bit result 64*64 bit multiplication
2. 64-bit addition with carry

In the past we implemented a version with 32-bit limbs, but as a result, when we accumulated limb products we always had to split 64-bit results of 32-bit multiplication back into 32-bit chunks. Had we not, the addition of 2 64-bit products would have lost the carry flag and the result would be incorrect. There were 2 issues with this:
1. This spawned in a lot of masking operations
2. We didn't use more efficient algorithms for squaring, because multiplication by 2 of intermediate products would once again overflow.

Switching to 9 29-bit limbs increased the number of multiplications from 136 to 171. However, since the product of 2 limbs is 58 bits, we can safely accumulate 64 of those before we have to reduce. This allowed us to get rid of a lot of intermediate masking operations, shifts and additions, so the resulting computation turned out to be more efficient. 

## Interaction of field object with other objects
Most of the time field is used with uint64_t or uint256_t in our codebase, but there is general logic of how we generate field elements from integers:
1. Converting from signed int takes the sign into account. It takes the absolute value, converts it to montgomery and then negates the result if the original value was negative
2. Unsigned integers ( <= 64 bits) are just converted to montgomery
3. uint256_t and uint512_t: 
    1. Truncate to 256 bits
    2. Subtract the modulus until the value is within field
    3. Convert to montgomery

Conversion from field elements exists only to unsigned integers and bools. The value is converted from montgomery and appropriate number of lowest bits is used to initialize the value. 

**N.B.** Functions for converting from uint256_t and back are not bijective, since values \f$ \ge p\f$ will be reduced.

## Field parameters

The field template is instantiated with field parameter classes, for example, class bb::Bn254FqParams. Each such class contains at least the modulus (in 64-bit and 29-bit form), r_inv (used to efficient reductions) and 2 versions of r_squared used for converting to Montgomery form (64-bit and WASM/29-bit version). r_squared and other parameters (such as cube_root, primitive_root and coset_generators) are defined for wasm separately, because the values represent an element already in Montgomery form.

## Helpful python snippets

Parse field parameters out of a parameter class (doesn't check and reconstitute endomorphism parameters, but checks correctness of everything else)
```python
import re
def parse_field_params(s):
    def parse_number(line):
        """Expects a string without whitespaces"""
        line=line.replace('U','').replace('L','') # Clear away all postfixes
        if line.find('0x')!=-1: # We have to parse hex
            value= int(line,16)
        else:
            value = int(line)
        return value

    def recover_single_value(name):
        nonlocal s
        index=s.find(name)
        if index==-1:
            raise ValueError("Couldn't find value with name "+name)
        eq_position=s[index:].find('=')
        line_end=s[index:].find(';')
        return parse_number(s[index+eq_position+1:index+line_end])        

    def recover_single_value_if_present(name):
        nonlocal s
        index=s.find(name)
        if index==-1:
            return None
        eq_position=s[index:].find('=')
        line_end=s[index:].find(';')
        return parse_number(s[index+eq_position+1:index+line_end])   

    def recover_array(name):
        nonlocal s
        index = s.find(name)
        number_of_elements=int(re.findall(r'(?<='+name+r'\[)\d+',s)[0])
        start_index=s[index:].find('{')
        end_index=s[index:].find('}')
        all_values=s[index+start_index+1:index+end_index]
        result=[parse_number(x) for (i,x) in enumerate(all_values.split(',')) if i<number_of_elements]
        return result

    def recover_multiple_arrays(prefix):
        chunk_names=re.findall(prefix+r'_\d+',s)
        recovered=dict()
        for name in chunk_names:
            recovered[name]=recover_array(name)
        return recovered

    def recover_element_from_parts(prefix,shift):
        """Recover a field element from its parts"""
        chunk_names=re.findall(prefix+r'_\d+',s)
        val_dict=dict()
        for name in chunk_names:
            val_dict[int(name[len(prefix)+1:])]=recover_single_value(name)
        result=0
        for i in range(len(val_dict)):
            result|=val_dict[i]<<(i*shift)
        return result

    def reconstruct_field_from_4_parts(arr):
        result=0
        for i, v in enumerate(arr):
            result|=v<<(i*64)
        return result
    parameter_dictionary=dict()
    parameter_dictionary['modulus']=recover_element_from_parts('modulus',64)
    parameter_dictionary['r_squared']=recover_element_from_parts('r_squared',64)
    parameter_dictionary['cube_root']=recover_element_from_parts('cube_root',64)
    parameter_dictionary['primitive_root']=recover_element_from_parts('primitive_root',64)

    parameter_dictionary['modulus_wasm']=recover_element_from_parts('modulus_wasm',29)
    parameter_dictionary['r_squared_wasm']=recover_element_from_parts('r_squared_wasm',64)
    parameter_dictionary['cube_root_wasm']=recover_element_from_parts('cube_root_wasm',64)
    parameter_dictionary['primitive_root_wasm']=recover_element_from_parts('primitive_root_wasm',64)
    parameter_dictionary={**parameter_dictionary,**recover_multiple_arrays('coset_generators')}
    parameter_dictionary={**parameter_dictionary,**recover_multiple_arrays('coset_generators_wasm')}
    parameter_dictionary['endo_g1_lo']=recover_single_value_if_present('endo_g1_lo')
    parameter_dictionary['endo_g1_mid']=recover_single_value_if_present('endo_g1_mid')
    parameter_dictionary['endo_g1_hi']=recover_single_value_if_present('endo_g1_hi')
    parameter_dictionary['endo_g2_lo']=recover_single_value_if_present('endo_g2_lo')
    parameter_dictionary['endo_g2_mid']=recover_single_value_if_present('endo_g2_mid')
    parameter_dictionary['endo_minus_b1_lo']=recover_single_value_if_present('endo_minus_b1_lo')
    parameter_dictionary['endo_minus_b1_mid']=recover_single_value_if_present('endo_minus_b1_mid')
    parameter_dictionary['endo_b2_lo']=recover_single_value_if_present('endo_b2_lo')
    parameter_dictionary['endo_b2_mid']=recover_single_value_if_present('endo_b2_mid')

    assert(parameter_dictionary['modulus']==parameter_dictionary['modulus_wasm']) # Check modulus representations are equivalent
    modulus=parameter_dictionary['modulus']
    r_wasm_divided_by_r_regular=2**(261-256)
    assert(parameter_dictionary['r_squared']==pow(2,512,modulus)) # Check r_squared
    assert(parameter_dictionary['r_squared_wasm']==pow(2,9*29*2,modulus)) # Check r_squared_wasm
    assert(parameter_dictionary['cube_root']*r_wasm_divided_by_r_regular%modulus==parameter_dictionary['cube_root_wasm'])
    assert(pow(parameter_dictionary['cube_root']*pow(2,-256,modulus),3,modulus)==1) # Check cubic root
    assert(pow(parameter_dictionary['cube_root_wasm']*pow(2,-29*9,modulus),3,modulus)==1) # Check cubic root for wasm
    assert(parameter_dictionary['primitive_root']*r_wasm_divided_by_r_regular%modulus==parameter_dictionary['primitive_root_wasm']) # Check pritimitve roots are equivalent
    for i in range(8):
        regular_coset_generator=reconstruct_field_from_4_parts([parameter_dictionary[f'coset_generators_{j}'][i] for j in range(4)])
        wasm_coset_generator=reconstruct_field_from_4_parts([parameter_dictionary[f'coset_generators_wasm_{j}'][i] for j in range(4)])
        assert(regular_coset_generator*r_wasm_divided_by_r_regular%modulus == wasm_coset_generator)

    return parameter_dictionary
```

Convert value from python to string for easy addition to bb's tests:
```python
def to_ff(value):
	print ("FF(uint256_t{"+','.join(["0x%xUL"%((value>>(i*64))&((1<<64)-1))for i in range(4)])+"})")
```