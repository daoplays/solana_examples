import numpy as np
from numpy import uint32

# return a length 32bit vector containing the binary representation of value
def value_to_binary(value):
	binary_values = np.zeros(32)
	bin_string = bin(value)[2:]
	n_bits = len(bin_string)
	start_bit = 32 - n_bits
	for i in range(start_bit, 32):
		binary_values[i] = int(bin_string[i - start_bit])

	return (binary_values).astype(int)
	
def  binary_vector_to_string(binary_vector):
	binary_string = "0b"
	for bit in binary_vector:
		binary_string += str(bit)
		
	return binary_string
	
def binary_to_value(binary_vector):
	
	binary_string  = binary_vector_to_string(binary_vector)	
	return int(binary_string, 2)
	
def make_shift_matrix(direction = "left", n = 1):

	L = np.zeros([32, 32])
	for i in range(0, 31):
		L[i + 1, i] = 1

	Ln =  L
	for i in range(n - 1):
		Ln = np.dot(L, Ln)
	
	if (direction == "left"):
		return Ln
	
	return Ln.T
	
	
def make_xor_left_shift_matrix(n = 1):
	
	L = make_shift_matrix("left", n)
		
	for i in range(0, 32):
		Ln[i , i] = 1
	
	return Ln
	
def make_xor_right_shift_matrix(n = 1):
	
	L = make_xor_left_shift_matrix(n)
	return L.T
	

value = 1234567890
binary_value = value_to_binary(value)

L1 = make_xor_left_shift_matrix(13)
R1 = make_xor_right_shift_matrix(17)
R2 = make_xor_right_shift_matrix(5)

LR_shifted = (np.dot(binary_value, np.dot(L1, np.dot(R1, R2)))%2).astype(int)

temp_value = uint32(value)
temp_value ^= uint32(temp_value << 13)
temp_value ^= uint32(temp_value >> 17)
temp_value ^= uint32(temp_value >> 5)

print(binary_to_value(LR_shifted), uint32(temp_value))


