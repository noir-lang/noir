import { CONTRACT_ADDRESS_PARAM_NAMES, pxe } from '../../config.js';
import { callContractFunction, deployContract, viewContractFunction } from '../../scripts/index.js';
import { convertArgs } from '../../scripts/util.js';
import styles from './contract_function_form.module.scss';
import { Button, Loader } from '@aztec/aztec-ui';
import { AztecAddress, CompleteAddress, Fr } from '@aztec/aztec.js';
import { ContractArtifact, FunctionArtifact } from '@aztec/foundation/abi';
import { useFormik } from 'formik';
import * as Yup from 'yup';

const DEFAULT_FIELD_VALUE = 100;
interface BasicParamDef {
  name: string;
  type: {
    kind: string;
    path?: string;
  };
}
interface ParamDef {
  name: string;
  type: {
    kind: string;
    path?: string;
    fields?: BasicParamDef[];
  };
}

type NoirFunctionYupSchema = {
  // hack: add `any` at the end to get the array schema to typecheck
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  [key: string]: Yup.NumberSchema | Yup.ArraySchema<number[], object> | Yup.BooleanSchema | any;
};

type NoirFunctionFormValues = {
  [key: string]: string | number | number[] | boolean | undefined;
};

// returns an object where first value is the yup type, second value is the default value
// this handles "base cases", which can be the parameters directly, or used
// in a recursive manner to handle structs
function generateYupDefaultValue(param: any, defaultAddress: string) {
  if (CONTRACT_ADDRESS_PARAM_NAMES.includes(param.name)) {
    // these are actually fields, which should be numbers, but yup doesn't support bigint so we convert back to bigint on execution
    return { yupType: Yup.string().required(), defaultValue: defaultAddress };
  } else if (param.type.kind === 'field') {
    return { yupType: Yup.number().required(), defaultValue: DEFAULT_FIELD_VALUE };
  } else if (param.type.kind === 'array') {
    const arrayLength = param.type.length;
    return {
      yupType: Yup.array()
        .of(Yup.number())
        .min(arrayLength)
        .max(arrayLength)
        .transform(function (value: number[], originalValue: string) {
          if (typeof originalValue === 'string') {
            return originalValue.split(',').map(Number);
          }
          return value;
        }),
      defaultValue: Array(arrayLength).fill(CONTRACT_ADDRESS_PARAM_NAMES.includes(param.name) ? defaultAddress : 200),
    };
  } else if (param.type.kind === 'boolean') {
    return { yupType: Yup.boolean().required(), defaultValue: false };
  } else {
    throw new Error('Unsupported type', param);
  }
}

function generateYupSchema(functionAbi: FunctionArtifact, defaultAddress: string) {
  const parameterSchema: NoirFunctionYupSchema = {};
  const initialValues: NoirFunctionFormValues = {};
  for (const param of functionAbi.parameters) {
    // use helper function for non struct-types
    if (['field', 'array', 'boolean'].includes(param.type.kind)) {
      const { yupType, defaultValue } = generateYupDefaultValue(param, defaultAddress);
      parameterSchema[param.name] = yupType;
      initialValues[param.name] = defaultValue;
      continue;
    } else if (param.type.kind === 'struct') {
      // for type checking, can't annotate left side of "for X of Y" statement
      const paramFields: ParamDef[] = param.type.fields!;
      const structParamSchema: any = {};
      const structInitialValues: any = {};
      for (const structParam of paramFields) {
        const { yupType, defaultValue } = generateYupDefaultValue(structParam, defaultAddress);
        structParamSchema[structParam.name] = yupType;
        structInitialValues[structParam.name] = defaultValue;
      }
      parameterSchema[param.name] = Yup.object().shape(structParamSchema);
      initialValues[param.name] = structInitialValues;
      continue;
    }
  }
  return { validationSchema: Yup.object().shape(parameterSchema), initialValues };
}

async function handleFunctionCall(
  contractAddress: AztecAddress | undefined,
  artifact: ContractArtifact,
  functionName: string,
  args: any,
  wallet: CompleteAddress,
) {
  const functionAbi = artifact.functions.find(f => f.name === functionName)!;
  const typedArgs: any[] = convertArgs(functionAbi, args);

  if (functionName === 'constructor' && !!wallet) {
    if (functionAbi === undefined) {
      throw new Error('Cannot find constructor in the ABI.');
    }
    // hack: addresses are stored as string in the form to avoid bigint compatibility issues with formik
    // convert those back to bigints before sending

    // for now, dont let user change the salt.  requires some change to the form generation if we want to let user choose one
    // since everything is currently based on parsing the contractABI, and the salt parameter is not present there
    const salt = Fr.random();
    return await deployContract(wallet, artifact, typedArgs, salt, pxe);
  }

  if (functionAbi.functionType === 'unconstrained') {
    return await viewContractFunction(contractAddress!, artifact, functionName, typedArgs, pxe, wallet);
  } else {
    const txnReceipt = await callContractFunction(contractAddress!, artifact, functionName, typedArgs, pxe, wallet);
    return `Transaction ${txnReceipt.status} on block number ${txnReceipt.blockNumber}`;
  }
}

interface ContractFunctionFormProps {
  wallet: CompleteAddress;
  contractAddress?: AztecAddress;
  artifact: ContractArtifact;
  functionAbi: FunctionArtifact;
  defaultAddress: string;
  title?: string;
  buttonText?: string;
  isLoading: boolean;
  disabled: boolean;
  onSubmit: () => void;
  onSuccess: (result: any) => void;
  onError: (msg: string) => void;
}

export function ContractFunctionForm({
  wallet,
  contractAddress,
  artifact,
  functionAbi,
  defaultAddress,
  buttonText = 'Submit',
  isLoading,
  disabled,
  onSubmit,
  onSuccess,
  onError,
}: ContractFunctionFormProps) {
  const { validationSchema, initialValues } = generateYupSchema(functionAbi, defaultAddress);
  const formik = useFormik({
    initialValues: initialValues,
    validationSchema: validationSchema,
    onSubmit: async (values: any) => {
      onSubmit();
      try {
        const result = await handleFunctionCall(contractAddress, artifact, functionAbi.name, values, wallet);
        onSuccess(result);
      } catch (e: any) {
        onError(e.message);
      }
    },
  });
  return (
    <form onSubmit={formik.handleSubmit} className={styles.content}>
      {functionAbi.parameters.map(input => (
        <div key={input.name} className={styles.field}>
          {input.type.kind !== 'struct' ? (
            <>
              <label className={styles.label} htmlFor={input.name}>
                {input.name} ({input.type.kind})
              </label>
              <input
                className={styles.input}
                id={input.name}
                name={input.name}
                disabled={isLoading}
                type="text"
                onChange={formik.handleChange}
                value={formik.values[input.name]}
              />
              {formik.touched[input.name] && formik.errors[input.name] && (
                <div>{formik.errors[input.name]?.toString()}</div>
              )}
            </>
          ) : (
            // Rendering object properties if the kind is 'struct'
            // find a better way to represent that these are part of the same input
            // than the text label `${input.name}.${field.name}`
            input.type.fields.map(field => (
              <div key={field.name}>
                <label className={styles.label} htmlFor={`${input.name}.${field.name}`}>
                  {`${input.name}.${field.name}`}
                </label>
                <input
                  className={styles.input}
                  id={`${input.name}.${field.name}`}
                  name={`${input.name}.${field.name}`}
                  disabled={isLoading}
                  type="text"
                  onChange={formik.handleChange}
                  value={formik.values[input.name] ? formik.values[input.name][field.name] : ''}
                />
                {/* {formik.touched[input.name] && formik.touched[input.name] && formik.errors[input.name] && formik.errors[input.name][field.name] && (
                <div>{formik.errors[input.name][field.name]?.toString()}</div>
              )} */}
              </div>
            ))
          )}
        </div>
      ))}
      {isLoading ? (
        <Loader />
      ) : (
        <Button disabled={disabled} text={buttonText} className={styles.actionButton} type="submit" />
      )}
    </form>
  );
}
