import random
import string
# import csv
import zlib

def generate_random_string(length):
    letters = string.hexdigits
    return ''.join(random.choice(letters) for _ in range(length))

def write_to_csv(filename, num, ticket_type ):
    # primes = [2, 3, 7]
    with open(filename, 'w') as csvfile:
        # writer = csv.writer(csvfile)
        for i in range(num):
            # suffix = random.choice(primes)
            # prefix = 11 - suffix
            random_string = '2:'  +  generate_random_string(2).lower() +  generate_random_string(20).lower() + ticket_type +\
                generate_random_string(2).lower() + str(4 - int(ticket_type))  + generate_random_string(4).lower() +  '\n'
            csvfile.write(random_string)

def main():
    # let typeMap = {
    #     0: "Early Bird 2-Day Ticket(4-5 May)",
    #     1: "Advance 2-Day Ticket(4-5 May)",
    #     2: "Regular 1-Day Ticket (4 May)",
    #     3: "Regular 1-Day Ticket (5 May)",
    #     4: "Regular 2-Day Ticket(4-5 May)",
    #     5: "Co-Host OKX WEB3(4-5 May)",
    #     6: "Fansland X OneKey(4-5 May)",
    # };

    # write_to_csv('qrcode_early.csv', num=5000, ticket_type='0' )
    # write_to_csv('qrcode_advance.csv', num=5000, ticket_type='1' )
    # write_to_csv('qrcode_regular_4_5.csv', num=10000, ticket_type='2' )
    # write_to_csv('qrcode_regular_5_5.csv', num=10000, ticket_type='3' )
    # write_to_csv('qrcode_regular_2_days.csv', num=20000, ticket_type='4')
    write_to_csv('qrcode_vvip_2_days.csv', num=4000, ticket_type='4')


    pass

if __name__ == '__main__':
    main()