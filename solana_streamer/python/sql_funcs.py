import sqlite3
from sqlite3 import Error

# setup the connection to the database and create the table if required
def create_connection():
	""" create a database connection to the SQLite database
	specified by db_file
	:param db_file: database file
	:return: Connection object or None
	"""
	
	db_file = r"signatures_db.db"
	conn = None
	try:
		conn = sqlite3.connect(db_file, isolation_level=None)
		
	except Error as e:
		print(e)
		return conn
		
	create_table(conn)

	return conn
	
# create the table with the structure required:
# the primary key is just the row index
# block_slot is the block the transaction is from
# choice is the value of the enum the user sent to the program
def create_table(conn):
	""" create a table from the create_table_sql statement
	:param conn: Connection object
	:return:
	"""
	
	create_signatures_table = """ CREATE TABLE IF NOT EXISTS signatures (
		id int PRIMARY_KEY,
		block_slot int NOT NULL,
		choice string NOT NULL); """
		
	try:
		c = conn.cursor()
		c.execute(create_signatures_table)
	except Error as e:
		print(e)
		
# inset a set of rows into the table within a single transaction
def insert_rows(conn, rows):
	"""
	Create a new entry in the signatures table
	:param conn:
	:param row:
	:return: project id
	"""
	sql = ''' INSERT INTO signatures(id,block_slot,choice)
	      VALUES(?,?,?) '''
	cur = conn.cursor()
	cur.execute("begin")
	for row in rows:
		cur.execute(sql, row)
	cur.execute("commit")
	
# get the next row id we  should use in the table
def get_next_row_id(conn):
	cur = conn.cursor()
	cur.execute("SELECT max(id) from signatures")
	r = cur.fetchone()
	if (r[0] == None):
		return 0
		
	return r[0]+1
	
def get_db_row(conn, row_id):
	cur = conn.cursor()
	cur.execute("SELECT * FROM signatures WHERE id=?", (row_id,))
	r = cur.fetchall()
	return r
	

