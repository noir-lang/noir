import lldb

# Define the bn254 modulus
N = 21888242871839275222246405745257275088548364400416034343698204186575808495617

# Define R as a power of 2 such that R > N (commonly used R for bn254 is 2^256)
R = 2**256

# Compute R inverse modulo N
R_inv = pow(R, -1, N)

def montgomery_to_standard(montgomery_value):
    # Convert from Montgomery form to standard representation
    standard_value = (montgomery_value * R_inv) % N
    return standard_value

def montgomery_summary(valobj, internal_dict):
    try:
        data = valobj.GetChildMemberWithName('data')
        data_0 = data.GetChildAtIndex(0).GetValueAsUnsigned()
        data_1 = data.GetChildAtIndex(1).GetValueAsUnsigned()
        data_2 = data.GetChildAtIndex(2).GetValueAsUnsigned()
        data_3 = data.GetChildAtIndex(3).GetValueAsUnsigned()

        montgomery_value = (
            data_0 +
            (data_1 << 64) +
            (data_2 << 128) +
            (data_3 << 192)
        )

        standard_value = montgomery_to_standard(montgomery_value)
        return hex(standard_value)
    except Exception as e:
        return f"Error: {e}"


def montgomery_summary2(valobj, internal_dict):
    return montgomery_summary(valobj.EvaluateExpression("get_value()"), internal_dict)


def __lldb_init_module(debugger, internal_dict):
    debugger.HandleCommand("type summary add --python-function lldb_format.montgomery_summary bb::fr")
    debugger.HandleCommand("type summary add --python-function lldb_format.montgomery_summary2 -x \"bb::stdlib::field_t.*\"")
    print('The "formatter" command has been installed!')
