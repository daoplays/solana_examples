from solana.rpc.api import Client
import concurrent.futures as cf

from sql_funcs import *
from rpc_funcs import *


conn = create_connection()
# check the connection is valid
if conn is None:
	print("Error! cannot create the database connection.")
	exit()

# connect to solana node
quick_node_dev = "YOUR_QUICK_NODE"
dev_client = Client(quick_node_dev)


current_row_id_to_insert = get_next_row_id(conn)


current_block = get_slot(dev_client)

print("Starting with row: ", current_row_id_to_insert, " Current block: ", current_block)
while(True):

	# get all the blocks after and including current_block
	block_list = get_block_list(dev_client, current_block)
	
	# if the last block in the list was the current block, just wait and check again shortly
	if(block_list[-1] == current_block):
		time.sleep(0.05)
		continue
	
	# we are only interested in the blocks after current_block so remove that one from the list
	block_list = block_list[1:]
	
	rows_to_insert = []
	# if there is only one block in the list we don't need to do any multithreading, just get the transactions and process them
	if(len(block_list) == 1):
	
		b_idx, n_t, data = get_data_from_block(dev_client, block_list[0])

		current_row_id_to_insert = create_rows_from_data(current_row_id_to_insert, b_idx, data, rows_to_insert)
	
	else:
	
		# if we have more than one block then multithread the requests and store them in a map with the block number as the key
		block_data = {}
		with cf.ThreadPoolExecutor(len(block_list)) as executor:
			futures = [executor.submit(get_data_from_block, dev_client, block_id) for block_id in block_list]
			
			for future in cf.as_completed(futures):
				# get the result for the next completed task
				b_result = future.result() # blocks
				block_data[b_result[0]] = b_result
		
		# once we have all the blocks process them in sequence so that they get stored in the database in sequential order
		for block_idx in block_list:

			b_idx, n_t, data = block_data[block_idx]

			current_row_id_to_insert = create_rows_from_data(current_row_id_to_insert, b_idx, data, rows_to_insert)
		
	insert_rows(conn, rows_to_insert)

	#  update current_block to the last one in our list
	current_block = block_list[-1]