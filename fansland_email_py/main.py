import time
import redis

from lib.sendemail import SendEmail, SendEmailMsg
from botocore.exceptions import ClientError

def main():
    r = redis.Redis(host='localhost', port=6379, db=0, password='gooDluck4u')
    email_client = SendEmail()

    chainid_maps = {
        # 主网
        '137': 'Polygon Mainnet',
        '56': 'BSC Mainnet ',
        '1': 'Ethereum Mainnet ',
        '42170': 'Arbitrum One Mainnet ',
        '10': 'OP Mainnet ',
        '43114': 'Avalanche C-Chain Mainnet ',

        # 测试网
        '80001': 'Polygon Mumbai Testnet',
        '97': 'BSC Testnet',
        '11155111': 'Ethereum Sepolia Testnet',
        '421614': 'Arbitrum Sepolia Testnet',
        '11155420': 'Op Sepolia Testnet',
        '43113': 'Avalanche Fuji Testnet'
    }

    typeid_maps = {
        '0': "Fansland Type 1",
        '1': "Fansland Type 2",
        '2': "Fansland Type 3",
        '3': "Fansland Type 4",
        '4': "Fansland Type 5",
        '5': "Fansland Type 6",
        '6': "Fansland Type 7",
    }


    while True:
        print('=================start send email ===============')
        try:
            raw_msg = r.lpop('sendemail')
            if raw_msg is  None:
                print('empty email queue')
            else:
                items = raw_msg.decode('utf-8').split(';')
                chainid = items[0]
                address = items[1]
                token_id = items[2]
                type_id = items[3]
                qrcode_txt = items[4]

                # 查一下redis该tokenid的最新owner
                user_address = r.get('nft:{}:nft:tokenid:owner:{}'.format(chainid, token_id))
                if user_address is None:
                    print("token_id {} owner is empty".format(token_id))
                    continue

                if user_address.decode('utf-8').lower() != str(address).lower():
                    print("token_id {} owner is not matched, expected: {}, got: ".format(
                        token_id, address, user_address.decode('utf-8').lower() ))
                    pass

                user_email = r.get('bindemail:{}'.format(address))
                if user_email is None:
                    print('address {} bind email is empty'.format(address))
                    continue
                user_email = user_email.decode('utf-8')

                chain = chainid_maps[chainid]
                print(user_email)

                response = email_client.send_raw_email(SendEmailMsg(
                        to_email=user_email,
                        qrcodes=[qrcode_txt],
                        address=address,
                        chain=chain,
                        chainid=chainid,
                        ticket_type = typeid_maps[str(type_id)]
                    ))

                print("Email sent! Message ID:"),
                print(response['MessageId'])
        except ClientError as e:
            print(e.response['Error']['Message'])
        except Exception as e:
            print("errror=========\n{}".format(e))

        time.sleep(5)

    pass


if __name__ == '__main__':
    main()

