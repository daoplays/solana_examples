import sqlite3
from sqlite3 import Error

N_COLS = 4

# setup the connection to the database and create the table if required
def create_database_connection():
    """ create a database connection to the SQLite database
    specified by db_file
    :param db_file: database file
    :return: Connection object or None
    """
	
    db_file = r"solana_block_data.db"
    conn = None
    try:
        conn = sqlite3.connect(db_file, isolation_level=None)
		
    except Error as e:
        print(e)
        return None
		
    success = create_table(conn)

    if (not success):
        return None

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
        choice string NOT NULL,
        bid_amount int NOT NULL); """
		
    try:
        c = conn.cursor()
        c.execute(create_signatures_table)
    except Error as e:
        print(e)
        return False

    return True
		
# inset a set of rows into the table within a single transaction
def insert_rows(conn, rows):
	"""
	Create a new entry in the signatures table
	:param conn:
	:param row:
	:return: project id
	"""
	sql = ''' INSERT INTO signatures(id,block_slot,choice,bid_amount)
	      VALUES(?,?,?,?) '''
	cur = conn.cursor()
	cur.execute("begin")
	for row in rows:
		cur.execute(sql, row)
	cur.execute("commit")
	
# returns the last row in the database, or None if it is empty
def get_last_db_row(conn):

    # get the row that has the maximum value of id
    # this returns a vector that has the shape [row, max_id]
    # so we only return the first N_COLS=4 values

    cur = conn.cursor()
    cur.execute("SELECT *, max(id) FROM signatures")
    r = cur.fetchone()

    if (r[0] == None):
        return None

    return r[:N_COLS]