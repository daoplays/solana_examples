import time
from borsh_construct import Enum, CStruct, U64
import base58
import requests
from requests.structures import CaseInsensitiveDict
import json as json
import numpy as np
import concurrent.futures as cf

# the enum listing available choices
choice_type = Enum(
	"A",
	"B",
	"C",
	"D",
    enum_name = "Choice"
)

# the structure that MakeChoice expects containing a choice and a bid amount
ChoiceData = CStruct(
    "choice" / choice_type,
    "bid_amount" / U64
)

# enum of instructions the program can be sent
message = Enum(
	"MakeChoice" / CStruct("choice_data" / ChoiceData),
	enum_name="ChoiceInstruction"
)


sleep_time = 0.25

def check_json_result(id, json_result):

    if ("result" in json_result.keys()):
        return True

    if ("error" in json_result.keys()):
        error = json_result["error"]
        print(id, " returned error: ", error)

    return False

# returns the current slot
def get_slot(dev_client):
    while True:
        try:
            slot = dev_client.get_slot()
        except:
            print("get_slot transaction request timed out")
            time.sleep(sleep_time)
            continue

        if (not check_json_result("get_slot", slot)):
            time.sleep(sleep_time)
            continue

        break
		
    return slot["result"]

# returns the list of finalized blocks after and including block_idx
def get_block_list(dev_client, current_block):

    print("requesting from block ", current_block)
    while True:
        try:
            block_list = dev_client.get_blocks(current_block)
        except:
            print("block_list transaction request timed out")
            time.sleep(sleep_time)
            continue

        if (not check_json_result("get_blocks", block_list)):
            time.sleep(sleep_time)
            continue

        if (len(block_list["result"]) == 0):
            time.sleep(sleep_time)
            continue          

        break

    return block_list["result"]

def make_blocks_batch_request(dev_client_url, block_list, have_block, blocks):
    headers = CaseInsensitiveDict()
    headers["Content-Type"] = "application/json"

    request_vec = []
    for i in range(len(block_list)):

        if (have_block[i]):
            continue

        new_request = json.loads('{"jsonrpc": "2.0","id": 0, "method":"getBlock", "params":[0, {"encoding": "json","transactionDetails":"full", "rewards": false, "maxSupportedTransactionVersion":0}]}')

        new_request["id"] = i + 1
        new_request["params"][0] = block_list[i]
        request_vec.append(new_request)

    while True:
        try:
            resp = requests.post(dev_client_url, headers=headers, data=json.dumps(request_vec))
        except:
            print("getBlock batch request timed out")
            time.sleep(sleep_time)
            continue

        break

    if (resp.status_code != 200):
        return have_block, blocks

    resp_json = resp.json()

    for response in resp_json:
        if ("id" not in response.keys()):
            continue

        if ("result" not in response.keys()):
            continue

        id = response["id"]
        blocks[block_list[id - 1]] = response["result"]
        have_block[id - 1] = True

    return have_block, blocks
    
def get_one_block_batch(dev_client_url, batch_block_list):
	
	batch_blocks = {}
	have_block = np.array([False] * len(batch_block_list))
	while (len(np.array(batch_block_list)[have_block == False]) != 0):
		print("requesting", len(batch_block_list), "blocks:", batch_block_list)
		have_block, batch_blocks = make_blocks_batch_request(dev_client_url, batch_block_list, have_block, batch_blocks)
		print(have_block)
			
	return batch_blocks
	
# Returns identity and transaction information about a confirmed block in the ledger
def get_blocks(dev_client_url, block_list):

    n_blocks = len(block_list)
    batch_size = 100
    # only submit max 100 requests in one go.  At some point this will start to timeout if too many are sent
    n_batches = n_blocks//batch_size + 1
    blocks = {}

    if (n_batches == 1):
        blocks = get_one_block_batch(dev_client_url, block_list)
    
    else:
        print("requesting ", n_batches, " with total ", n_blocks, " blocks")

        batch_lists = []
        for batch in range(n_batches):
            batch_start = batch * batch_size
            batch_end = min(n_blocks, batch_start + batch_size)
            batch_block_list = block_list[batch_start : batch_end]
            batch_lists.append(batch_block_list)
			
        max_threads = 10
        with cf.ThreadPoolExecutor(max_threads) as executor:
            futures = [executor.submit(get_one_block_batch, dev_client_url, batch_lists[batch_id]) for batch_id in range(n_batches)]
            
            for future in cf.as_completed(futures):
                # get the result for the next completed task
                batch_blocks = future.result() # blocks
                for block in batch_blocks.keys():
                    blocks[block] = batch_blocks[block]
		
    return blocks


# get the block and process it
def get_data_from_block(block_idx, block):

    data_vec = []

    program = "H73oSXtdJfuBz8JWwdqyG92D3txMqxPEhAhT23T8eHf5"

    for t in block["transactions"]:
        transaction_message = t["transaction"]["message"]
        accounts = transaction_message["accountKeys"]
        instructions = transaction_message["instructions"]

        for instruction in instructions:

            program_index = instruction["programIdIndex"]

            if (program_index >= len(accounts)):
                continue

            if (accounts[program_index] != program):
                continue
            
            if ("data" not in instruction.keys()):
                continue

            data = instruction["data"]
            decoded_data = base58.b58decode(data)

            try:
                args = message.parse(decoded_data)
            except:
                print("unable to parse data", decoded_data)
                continue

            if(not isinstance(args, message.enum.MakeChoice)):
                print("Have data but not a MakeChoice:", args)
                continue
            
            data_vec.append(args)

    return block_idx, data_vec	


# create the rows for the database from the block data
def create_rows_from_data(row_id_to_insert, block_id, data, rows_vec):

    if(len(data) == 0):
        new_row = (row_id_to_insert, block_id, "no_choice", 0)
        print("adding row: ", new_row)
        rows_vec.append(new_row)
        row_id_to_insert += 1
    else:
        for i in range(len(data)):
            args = data[i]
            row_id = row_id_to_insert + i
            new_row = (row_id, block_id, str(args.choice_data.choice), args.choice_data.bid_amount)
            print("adding row: ", new_row)
            rows_vec.append(new_row)
			
        row_id_to_insert += len(data)
			
    return row_id_to_insert