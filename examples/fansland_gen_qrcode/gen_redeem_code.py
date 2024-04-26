import random
import string
# import csv

def generate_random_string(length):
    letters = 'ABCDEFGHJKLMNPRSTUVWXYZ23456789'
    return ''.join(random.choice(letters) for _ in range(length))

def write_to_csv(filename, num, ticket_type, ):
    with open(filename, 'w') as csvfile:
        # writer = csv.writer(csvfile)
        for _ in range(num):
            random_string =  generate_random_string(9).upper() + ',' +  ticket_type + '\n'
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

    # write_to_csv('qrcode.csv', num=5000, ticket_type='Early Bird 2-Day Ticket(4-5 May)' )
    # write_to_csv('qrcode.csv', num=5000, ticket_type='Advance 2-Day Ticket(4-5 May)' )
    # write_to_csv('qrcode.csv', num=20000, ticket_type='Regular 1-Day Ticket (4 May)' )
    # write_to_csv('qrcode.csv', num=20000, ticket_type='Regular 1-Day Ticket (5 May)' )
    # write_to_csv('qrcode.csv', num=20000, ticket_type='Regular 2-Day Ticket(4-5 May)' )
    # write_to_csv('redeem_code_vvip_1000.csv', num=1000, ticket_type='4' )
    write_to_csv('redeem_code_ime_100_4_5.csv', num=100, ticket_type='Regular 1-Day Ticket (4 May)' )
    write_to_csv('redeem_code_ime_100_5_5.csv', num=100, ticket_type='Regular 1-Day Ticket (5 May)' )

    pass

if __name__ == '__main__':
    main()