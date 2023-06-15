import time
from borsh_construct import Enum, CStruct, String, U64, U8
import base58
import requests
from requests.structures import CaseInsensitiveDict
import json as json
import numpy as np
import concurrent.futures as cf
import datetime
from log import *


Twitter_Instructions = Enum(
    "InitProgram" / CStruct("supporter_amount" / U64),
    "Register" / CStruct("tweet_id" / U64),
    "CreateUserAccount",
    "NewFollower"/ CStruct("user_id" / U64),
    "SetError"/ CStruct("error_code" / U8),
    "CheckFollower",
    "CheckHashTag"/ CStruct("tweet_id" / U64, "hashtag" / String),
    "SendTokens"  / CStruct("amount" / U64, "tweet_id" / U64, "hashtag" / String),
    "CheckRetweet"/ CStruct("tweet_id" / U64, "hashtag" / String),
    "SetUserID" / CStruct("user_id" / U64),
    enum_name="TwitterInstruction", 
)


program_key = "4jvaAM7NpyXxFHjELkkEAMQ7jUPe9FuA6kUqj2FSMuHS"
sleep_time = 0.25


def check_json_result(id, json_result):

	if ("error" in json_result.keys()):
		error = json_result["error"]
		log_error(str(id) + " returned error: " + str(error))
		return False

	if ("result" in json_result.keys()):
		return True

	return False

# returns the current slot
def get_slot(dev_client):
    while True:
        try:
            slot = dev_client.get_slot()
        except:
            log_error("get_slot transaction request timed out")
            time.sleep(sleep_time)
            continue

        if (not check_json_result("get_slot", slot)):
            time.sleep(sleep_time)
            continue

        break
		
    return slot["result"]




# returns the list of finalized blocks after and including block_idx
def sub_get_transactions(dev_client_url, signatures, have_transactions, transactions):


    headers = CaseInsensitiveDict()
    headers["Content-Type"] = "application/json"

    data_vec = []
    for i in range(len(signatures)):

        if (have_transactions[i]):
            continue

        new_request = json.loads('{"jsonrpc": "2.0","id": 1, "method":"getTransaction", "params":["GRxdexptfCKuXfGpTGREEjtwTrZPTwZSfdSXiWDC11me", {"encoding": "json", "maxSupportedTransactionVersion":0, "commitment": "confirmed"}]}')

        new_request["id"] = i + 1
        new_request["params"][0] = signatures[i]
        data_vec.append(new_request)
   
    print("submit transactions post request")
    while True:
        try:
            resp = requests.post(dev_client_url, headers=headers, data=json.dumps(data_vec), timeout=10)
        except:
            log_error("getTransaction request timed out")
            time.sleep(sleep_time)
            continue

        if (resp.status_code != 200):
            log_error("getTransaction request unsuccessful")
            time.sleep(sleep_time)
            continue


        break

    if (resp.status_code != 200):
        return have_transactions, transactions

    response_json = resp.json()

    for response in response_json:
        if ("id" not in response.keys()):
            log_error("No id in getTransaction response")
            continue

        if ("error" in response.keys()):
            log_error("error in getTransaction response: " + str(response["error"]))
            continue

        if ("result" not in response.keys()):
            log_error("No result in getTransaction response")
            continue

        if (response["result"] == None):
            log_error("result is None in getTransaction response")
            continue

        id = response["id"]
        transactions[signatures[id - 1]] = response["result"]
        have_transactions[id - 1] = True

    
    return have_transactions, transactions

def get_one_transaction_batch(dev_client_url, signatures):
	
	batch_transactions = {}
	have_transactions = np.array([False] * len(signatures))
	while (len(np.array(signatures)[have_transactions == False]) != 0):
		log_info("requesting " + str(len(np.array(signatures)[have_transactions == False]))  + " transactions")
		have_transactions, batch_transactions = sub_get_transactions(dev_client_url, signatures, have_transactions, batch_transactions)
		print(have_transactions)
			
	return batch_transactions


def get_transactions(dev_client_url, signatures):

    n_sigs = len(signatures)
    batch_size = 100
    # only submit max 100 requests in one go.  At some point this will start to timeout if too many are sent
    n_batches = n_sigs//batch_size + 1
    transactions = []

    if (n_batches == 1):
        batch_results = get_one_transaction_batch(dev_client_url, signatures)
    
    else:
        log_info("requesting " + str(n_batches) + " with total " + str(n_sigs) + " signatures")

        batch_lists = []
        for batch in range(n_batches):
            batch_start = batch * batch_size
            batch_end = min(n_sigs, batch_start + batch_size)
            batch_block_list = signatures[batch_start : batch_end]
            batch_lists.append(batch_block_list)
			
        batch_results = {}
        with cf.ThreadPoolExecutor(10) as executor:
            futures = [executor.submit(get_one_transaction_batch, dev_client_url, batch_lists[batch_id]) for batch_id in range(n_batches)]

            for future in cf.as_completed(futures):
                # get the result for the next completed task
                batch_transactions = future.result() # blocks
                for key in batch_transactions.keys():
                    batch_results[key] = batch_transactions[key]

        
    for key in batch_results.keys():
            transactions.append(batch_results[key])
		
    return transactions

def perform_request_response_checks(function_name, response_json):

    if ("id" not in response_json.keys()):
        log_error("No id in " + function_name + " response")
        return False

    if ("error" in response_json.keys()):
        log_error("error in " + function_name + " response " + str(response_json["error"]))
        return False

    if ("result" not in response_json.keys()):
        log_error("No result in " + function_name + " response")
        return False

    if (response_json["result"] == None):
        log_error("result is None in " + function_name + " response")
        return False

    return True

def get_request_header():

    headers = CaseInsensitiveDict()
    headers["Content-Type"] = "application/json"
    headers["x-session-hash"] = "blabla"

    return headers

# returns the json request for getSignaturesForAddress
def get_signatures_request(current_signature = None, id = 1):


    new_request = json.loads('{"jsonrpc": "2.0","id": 1, "method":"getSignaturesForAddress", "params":["11111111111111111111111111111111", {"commitment": "confirmed"}]}')

    new_request["params"][0] = program_key
    new_request["id"] = id
    if (current_signature != None):
        new_request["params"][1]["until"] = current_signature

    return new_request, get_request_header()

# returns the list of finalized blocks after and including block_idx
def get_signatures(dev_client_url, min_slot = 0, max_slot = np.inf, current_signature=None):

    if (current_signature != None):
        log_info("requesting from signature " + str(current_signature))

    new_request, headers = get_signatures_request(current_signature)

   
    while True:
        try:
            resp = requests.post(dev_client_url, headers=headers, data=json.dumps([new_request]), timeout=10)
        except:
            log_error("getSignaturesForAddress request timed out")
            time.sleep(sleep_time)
            continue

        if (resp.status_code != 200):
            log_error("getSignaturesForAddress request unsuccessful, try again")
            time.sleep(sleep_time)
            continue

        response_json = resp.json()[0]

        if (not perform_request_response_checks("getSignaturesForAddress", response_json)):
            time.sleep(sleep_time)
            continue

        break

    #print(response_json)
    result = response_json["result"]

    log_info("have " + str(len(result)) + " signatures")

    # we need to update current_signature to the latest one whose slot doesn't exceed max_slot
    if (len(result) > 0):
        for r in result:
            if r["slot"] > max_slot:
                continue

            current_signature = r["signature"]
            break

    signatures = []
    for r in result:

        if(r["slot"] < min_slot or r["slot"] > max_slot):
            print("sig outside slots: ", min_slot, max_slot, datetime.datetime.utcnow(), r)
            continue

        print(datetime.datetime.utcnow(), r)
        signatures.append(r["signature"])
        
    return signatures, current_signature


# get the block and process it
def get_data_from_transaction(transaction):

    data_vec = []

    transaction_message = transaction["message"]
    accounts = transaction_message["accountKeys"]

    for instruction in transaction_message["instructions"]:

        program_index = instruction["programIdIndex"]

        if (accounts[program_index] != program_key):
            continue
        
        if ("data" not in instruction.keys()):
            continue

        data = instruction["data"]
        decoded_data = base58.b58decode(data)
        result = {}
        try:
            args = Twitter_Instructions.parse(decoded_data)
        except:
            log_error("unable to parse data: " + str(decoded_data))
            continue

        if(isinstance(args, Twitter_Instructions.enum.InitProgram)):
            print("Have data but is from InitProgram:", args)
            continue

        if(isinstance(args, Twitter_Instructions.enum.SetError)):
            print("Have data but is from SetError:", args)
            continue

        if(isinstance(args, Twitter_Instructions.enum.CreateUserAccount)):
            print("Have data but is from CreateUserAccount:", args)
            continue

        if(isinstance(args, Twitter_Instructions.enum.NewFollower)):
            print("Have data but is from NewFollower:", args)
            continue

        result["args"] = args
        if(isinstance(args, Twitter_Instructions.enum.Register)):
            user_idx = instruction["accounts"][0]
            if (user_idx < len(accounts)):
                user_account = accounts[user_idx]

            print(user_account)
            result["user"] = user_account

        if (isinstance(args, Twitter_Instructions.enum.CheckHashTag)):
            user_idx = instruction["accounts"][0]
            if (user_idx < len(accounts)):
                user_account = accounts[user_idx]

            print(user_account)
            result["user"] = user_account

        if (isinstance(args, Twitter_Instructions.enum.CheckRetweet)):
            user_idx = instruction["accounts"][0]
            if (user_idx < len(accounts)):
                user_account = accounts[user_idx]

            print(user_account)
            result["user"] = user_account

        if (isinstance(args, Twitter_Instructions.enum.CheckFollower)):
            user_idx = instruction["accounts"][0]
            if (user_idx < len(accounts)):
                user_account = accounts[user_idx]

            print(user_account)
            result["user"] = user_account
      
        data_vec.append(result)

    return data_vec	

