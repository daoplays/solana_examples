import time
from borsh_construct import Enum, CStruct, U64
import base58

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
	
# Returns identity and transaction information about a confirmed block in the ledger
def get_block(dev_client, block_idx):
    while True:
        try:
            block = dev_client.get_block(block_idx)
        except:
            print("get_block request timed out")
            time.sleep(sleep_time)
            continue

        if (not check_json_result("get_block", block)):
            time.sleep(sleep_time)
            continue

        break
		
    return block["result"]

# get the block and process it
def get_data_from_block(dev_client, block_idx):

    block = get_block(dev_client, block_idx)

    data_vec = []
    program = "H73oSXtdJfuBz8JWwdqyG92D3txMqxPEhAhT23T8eHf5"

    for t in block["transactions"]:
        transaction_message = t["transaction"]["message"]
        if(program not in transaction_message["accountKeys"]):
            continue

        for instruction in transaction_message["instructions"]:
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
