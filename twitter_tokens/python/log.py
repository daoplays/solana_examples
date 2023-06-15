import datetime

class bcolors:
    HEADER = '\033[95m'
    OKBLUE = '\033[94m'
    OKCYAN = '\033[96m'
    OKGREEN = '\033[92m'
    WARNING = '\033[93m'
    FAIL = '\033[91m'
    ENDC = '\033[0m'
    BOLD = '\033[1m'
    UNDERLINE = '\033[4m'
    
def log(string):
    print(str(datetime.datetime.utcnow()) + " " + string)

def log_error(string):
    print(str(datetime.datetime.utcnow()) + bcolors.FAIL + " ERROR: " + string + bcolors.ENDC)
		
def log_info(string):
	print(str(datetime.datetime.utcnow()) + bcolors.OKBLUE + " INFO: " + string + bcolors.ENDC)

def log_db(string):
	print(str(datetime.datetime.utcnow()) + bcolors.OKGREEN + " DB: " + string + bcolors.ENDC)