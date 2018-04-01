export const newObject = () => ({});
export const emitUsize = (obj: object, key: string, val: number) => obj[key] = val;
export const emitIsize = (obj: object, key: string, val: number) => obj[key] = val;
export const emitBool = (obj: object, key: string, val: boolean) => obj[key] = val;
export const emitChar = (obj: object, key: string, val: number) => obj[key] = String.fromCharCode(val);
export const emitU8 = (obj: object, key: string, val: number) => obj[key] = val;
export const emitI8 = (obj: object, key: string, val: number) => obj[key] = val;
export const emitU16 = (obj: object, key: string, val: number) => obj[key] = val;
export const emitI16 = (obj: object, key: string, val: number) => obj[key] = val;
export const emitU32 = (obj: object, key: string, val: number) => obj[key] = val;
export const emitI32 = (obj: object, key: string, val: number) => obj[key] = val;
export const emitF32 = (obj: object, key: string, val: number) => obj[key] = val;
export const emitU64 = (obj: object, key: string, val: number) => obj[key] = val;
export const emitI64 = (obj: object, key: string, val1: number, val2: number) => obj[key] = (val1 << 32) | val2;
export const emitF64 = (obj: object, key: string, val: number) => obj[key] = val;
export const emitStr = (obj: object, key: string, val: string) => obj[key] = val;
export const emitUnit = (obj: object, key: string) => obj[key] = null;
export const emitNone = (obj: object, key: string) => obj[key] = null;
