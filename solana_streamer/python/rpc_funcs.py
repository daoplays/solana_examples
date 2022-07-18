from datetime import datetime
from solana.rpc.api import Client
import time
import concurrent.futures as cf
from borsh_construct import Enum, CStruct, U32, U64, Vec
from enum import IntEnum
import base58
import numpy as np

# the enum listing available choices
choice_type = Enum(
	"A",
	"B",
	"C",
	"D",
    	enum_name = "Choice"
)

BlockChoices = CStruct(
    "num_choices" / U32,
    "choices" / Vec(choice_type),
    "weights"  / Vec(U64)
)


ChoiceData = CStruct(
    "blocks_to_process" / U32,
    "all_choices" / Vec(BlockChoices)
)

# enum of instructions the program can be sent
message = Enum(
	"InitDataAccount",
	"MakeChoice" / CStruct("choice" / choice_type),
	"HandleChoices" / CStruct("choice_data" / ChoiceData),

	enum_name="ChoiceInstruction"
)

# returns the current slot
def get_slot(dev_client):
	while True:
		try:
			slot = dev_client.get_slot()
		except:
			print("get_slot transaction request timed out")
			time.sleep(1)
			continue
			
		if("result" not in slot.keys()):
			print("get_slot has no result key")
			time.sleep(1)
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
			time.sleep(1)
			continue
			
		if("result" not in block_list.keys()):
			print("block_list has no result key")
			time.sleep(1)
			continue
			
		break
		
	return block_list["result"]
	
# Returns identity and transaction information about a confirmed block in the ledger
def get_block(dev_client, block_idx):
	while True:
		try:
			block = dev_client.get_block(block_idx)
		except:
			print("transaction request timed out")
			time.sleep(1)
			continue
		break
		
	return block
	

# go through all the transactions in the block and extract the data from the relevant ones
def process_block(block):

	data = []
	program = "49tLGpRt6ikpsGrWQ5ke5QJcFjJGqrbkNgLgUvK8gBKS"
	n_t = len(block["transactions"])
	for t in block["transactions"]:
		message = t["transaction"]["message"]
		if(program not in message["accountKeys"]):
			continue
			
		if(len(message["instructions"]) > 1):
			print("invalid set of instructions")
			print(t)
			continue
			
		data.append(message["instructions"][0]["data"])
		
	return n_t, data

# get the block and process it
def get_data_from_block(dev_client, block_idx):

	#print("trying block: ", block_idx)
	while(True):
		block = get_block(dev_client, block_idx)
		if("result" in block.keys()):
			break

		if("error" not in block.keys()):
			print("no error or result!: ", block)
			break
			
		error = block["error"]
		print("Try again:", error)
		time.sleep(0.1)
		
	
	n_t, data = process_block(block["result"])
	return block_idx, n_t, data
	
# create the rows for the database from the block data
def create_rows_from_data(row_id_to_insert, block_id, data, rows_vec):

	if(len(data) == 0):
		new_row = (row_id_to_insert, block_id, "no_choice")
		print("adding row: ", new_row)
		rows_vec.append(new_row)
		row_id_to_insert += 1
	else:
		for i in range(len(data)):
			entry = data[i]
			row_id = row_id_to_insert + i
			decoded_data = base58.b58decode(entry)
			try:
				args = message.parse(decoded_data)
			except:
				print("unable to parse data", decoded_data)
				continue
				
			if(not isinstance(args, message.enum.MakeChoice)):
				print("Have data but not a MakeChoice:", args)
				continue
				
			new_row = (row_id, block_id, str(args.choice))
			print("adding row: ", new_row)
			rows_vec.append(new_row)
			
		row_id_to_insert += len(data)
			
	return row_id_to_insert
