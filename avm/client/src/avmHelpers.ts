import { CallResultsArray, InterpreterResult, CallRequest } from './types';

const decoder = new TextDecoder();
const encoder = new TextEncoder();

export function prepareArgs(
    air: string,
    prevData: Uint8Array,
    data: Uint8Array,
    params: {
        initPeerId: string;
        currentPeerId: string;
    },
    callResults: CallResultsArray,
): string {
    const callResultsToPass: any = {};
    for (let [k, v] of callResults) {
        callResultsToPass[k] = {
            ret_code: v.retCode,
            result: v.result,
        };
    }

    const paramsToPass = {
        init_peer_id: params.initPeerId,
        current_peer_id: params.currentPeerId,
    };

    const encoded = encoder.encode(JSON.stringify(callResultsToPass));

    const avmArg = JSON.stringify([
        // force new line
        air,
        Array.from(prevData),
        Array.from(data),
        paramsToPass,
        Array.from(encoded),
    ]);

    return avmArg;
}

export function convertInterpreterResult(rawResult: string): InterpreterResult {
    let result: any;
    try {
        result = JSON.parse(rawResult);
    } catch (ex) {
        throw 'call_module result parsing error: ' + ex + ', original text: ' + rawResult;
    }

    if (result.error !== '') {
        throw 'call_module returned error: ' + result.error;
    }

    result = result.result;

    const callRequestsStr = decoder.decode(new Uint8Array(result.call_requests));
    let parsedCallRequests;
    try {
        if (callRequestsStr.length === 0) {
            parsedCallRequests = {};
        } else {
            parsedCallRequests = JSON.parse(callRequestsStr);
        }
    } catch (e) {
        throw "Couldn't parse call requests: " + e + '. Original string is: ' + callRequestsStr;
    }

    let resultCallRequests: Array<[key: number, callRequest: CallRequest]> = [];
    for (const key in parsedCallRequests) {
        const callRequest = parsedCallRequests[key];

        let arguments_;
        let tetraplets;
        try {
            arguments_ = JSON.parse(callRequest.arguments);
        } catch (e) {
            throw "Couldn't parse arguments: " + e + '. Original string is: ' + arguments_;
        }

        try {
            tetraplets = JSON.parse(callRequest.tetraplets);
        } catch (e) {
            throw "Couldn't parse tetraplets: " + e + '. Original string is: ' + tetraplets;
        }

        resultCallRequests.push([
            key as any,
            {
                serviceId: callRequest.service_id,
                functionName: callRequest.function_name,
                arguments: arguments_,
                tetraplets: tetraplets,
            },
        ]);
    }
    return {
        retCode: result.ret_code,
        errorMessage: result.error_message,
        data: result.data,
        nextPeerPks: result.next_peer_pks,
        callRequests: resultCallRequests,
    };
}

type FaaSCall = ((args: string) => Promise<string>) | ((args: string) => string);

export async function runAvm(
    faasCall: FaaSCall,
    air: string,
    prevData: Uint8Array,
    data: Uint8Array,
    params: {
        initPeerId: string;
        currentPeerId: string;
    },
    callResults: CallResultsArray,
): Promise<InterpreterResult> {
    try {
        const avmArg = prepareArgs(air, prevData, data, params, callResults);
        const rawResult = await faasCall(avmArg);
        return convertInterpreterResult(rawResult);
    } catch (e) {
        return {
            retCode: -1,
            errorMessage: 'marine-js call failed, ' + e,
        } as any;
    }
}
