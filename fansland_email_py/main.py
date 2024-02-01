import time
import redis

from lib.sendemail import SendEmail, SendEmailMsg
from botocore.exceptions import ClientError

def main():
    r = redis.Redis(host='localhost', port=6379, db=0, password='gooDluck4u')
    email_client = SendEmail()

    while True:
        try:
            email_addr = r.lpop('sendemail')
            if email_addr is  None:
                print('empty email queue')
            else:
                # TODO: parse email content
                qrcode = "1:xxxxyyyyyyyyaaaaaaaaaaaalllllllll"
                address = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                chain = "polygon"
                print(email_addr)

                response = email_client.send_raw_email(SendEmailMsg(
                        to_email=email_addr.decode('utf-8'),
                        qrcodes=[qrcode],
                        address=address,
                        chain=chain
                    ))

                print("Email sent! Message ID:"),
                print(response['MessageId'])

            time.sleep(2)
        except ClientError as e:
            print(e.response['Error']['Message'])
            print("errror=========\n{}".format(e))

    pass


if __name__ == '__main__':
    main()

