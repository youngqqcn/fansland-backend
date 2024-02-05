

# def get_mint_template(address: str, img_count: int) :
#     first_part = f"""<div style=width:100%; background-color:#E4E5E7;padding:2px;>
#         <div style=max-width:640px; margin:0 auto; background:#ffffff;border-radius:10px; overflow:hidden;>
#             <img src=https://static.fansland.io/email/20240126-114328.jpg style=width:100%  alt='' />
#             <div style=padding:32px;>
#                 <h1 style=font-size:24px; font-weight:bold; text-align:center; color:#081131;> Fansland NFT ticket Mint success! 🎉 </h1>
#                 <p style=margin:0;><br></p>
#                 <p style=font-size:15px; line-height:170%; letter-spacing:0.2px;  margin:0;> Congratulations! 👏</p>
#                 <p style=margin:0;><br></p>
#                 <p style=font-size:15px; line-height:170%; letter-spacing:0.2px; margin:0;> You have successfully mint Fansland NFT tickets with your wallet address: </p>
#                 <p style=margin:0;><br></p>
#                 <p style=font-size:15px; text-align:center;  line-height:170%; letter-spacing:0.2px; margin:0;> {address} </p>
#                 <p style=margin:0;><br></p>
#                 <p style=font-size:15px; line-height:170%; letter-spacing:0.2px; margin:0;>
#                     Please login to <a href="https://fansland.io/">https://fansland.io</a> with this wallet address to see your NFT tickets.
#                 </p>
#                 <p style=margin:0;><br></p>
#                 <hr>
#                 <p style=font-size:14px; font-style: italic;  text-align:center;><em> Fansland aims to build the best Web3 infrastructure for global fans.<em> </p>
#                 """

#     img_part = ""
#     for i in range(img_count):
#         img_part += f"""<img src="cid:qrcode{i}" alt="qrcode{i}">"""
#         img_part += f"""<p style=margin:0;><br></p>"""

#     last_part = """
#             </div>
#         </div>
#     </div>"""

#     return first_part + img_part + last_part


def get_mint_template(chain, chainid, address, ticket_type, token_id) :
    location = "Bangkok,Thailand"
    ticketType = ticket_type
    date = "4-6 May 2024"
    first_part = f"""
<div style=width:100%; background-color:#E4E5E7;padding:2px;>
    <div style="max-width:640px; margin:0 auto; background:#ffffff;border-radius:10px; overflow:hidden;">
        <img src=https://static.fansland.io/email/20240126-114328.jpg style=width:100%  alt='' />
        <div style="padding:12px;">
            <h1 style="font-size:18px; font-weight:bold; text-align:center; color:black;"> Fansland NFT ticket Mint success! 🎉 </h1>
            <p style="margin:0;"><br></p>
            <p style="font-size:14px;  line-height:110%;    margin:0; color:black;"> Congratulations! 👏</p>
            <p style="margin:0;"><br></p>
            <p style="font-size:14px;  line-height:110%;   margin:0; color:black;"> You have successfully mint Fansland NFT ticket.</p>
            <p style="margin:0;"><br></p>
            <p style="font-size:14px;  line-height:110%;   margin:0; color:black;">
               It is a privilege that you will be attending Fansland Web3 Music Festival.
            </p>
            <p style="margin:0;"><br></p>
            <p style="font-size:14px;  line-height:110%;   margin:0; color:black;">
               Now, we are delighted to provide you with the following details about your NFT ticket:
            </p>
            <ul style="padding-inline-start:20px;">
                <li>
                    <p style="font-size:14px;    line-height:170%;   margin:0; color:black;">
                    🎫 Ticket Type: <span style="font-weight: 600;">{ticketType}</span>
                    </p>
                </li>
                <li>
                    <p style="font-size:14px;    line-height:170%;   margin:0;color:black;">
                    🔗 Blockchain: <span style="font-weight: 600;">{chain} ({chainid})</span>
                    </p>
                </li>
                <li>
                    <p style="font-size:14px;    line-height:170%;   margin:0;color:black;">
                    👤 Wallet Address: <span style="font-weight: 600;">{address}</span>
                    </p>
                </li>
                <li>
                    <p style="font-size:14px;    line-height:170%;   margin:0;color:black;">
                    🆔 NFT Name: <span style="font-weight: 600;"> Fansland #{token_id}</span>
                    </p>
                </li>
                <li>
                    <p style="font-size:14px;    line-height:170%;   margin:0;color:black;">
                    🗓️ Event Date: <span style="font-weight: 600;">{date}</span>
                    </p>
                </li>
                <li>
                    <p style="font-size:14px;    line-height:170%;   margin:0;color:black;">
                    🌏 Event Location: <span style="font-weight: 600;">{location}</span>
                    </p>
                </li>
                <li>
                    <p style="font-size:14px;    line-height:170%;   margin:0;color:black;">
                     💾 Check-in QR Code: <span style="font-weight: 600;"> </span>
                    </p>
                </li>
            </ul>
            <p style="text-align: center;">
              <img src="cid:qrcode" alt="qrcode">
            </p>
            <p style="font-size:13px; text-align:center; margin:0;color:black;">
                (Please keep safely, do not transfer it to others)
            </p>
            <p style="margin:0;"><br></p>
            <p style="text-align: center;">
              <a href="https://fansland.io" target="_blank">
                <button style="border: none; color: #FFF !important; font-size: 14px; font-weight: 600; border-radius: 12px; padding: 12px 24px; background: linear-gradient(251deg, #AE18FD 8%, #FC6512 90%);">View Event Details</button>
              </a>
            </p>
            <p style="margin:0;"><br></p>
            <p style="font-size:14px;  line-height:110%;   margin:0;color:black;">
                You could also login to <a href="https://fansland.io/">https://fansland.io</a> with wallet to view your NFT tickets.
            </p>
            <p style="margin:0;"><br></p>
            <p style="margin:0;"><br></p>
            <p style="font-size:14px;  line-height:110%;   margin:0;color:rgb(131, 126, 126);">
                <em>This is an automated message, please do not reply. If you have any questions, please email our support team at <a href="mailto:service@fansland.io">service@fansland.io</a></em>
            </p>
            <p style="margin:0;"><br></p>
            <hr>
            <p style="text-align: center;">
              <img style="width: 180px;" src="https://static.fansland.io/email/fansland.png" />
            </p>
            <p style="text-align: center; font-size:13px;  line-height:100%;  color:black;">
              Build The Best Web3 Infrastructure for Global FANS
            </p>
            <p style="text-align: center;">
              <a style="margin:0;text-decoration: none;" href="https://medium.com/@fansland" target="_blank">
                <img style="border-radius: 0%;height:20px; width: 20px; overflow: hidden;"  src="https://static.fansland.io/email/twitter.png" alt="">
              </a>
              <a style="margin:0;text-decoration: none;" href="https://medium.com/@fansland" target="_blank">
                <img style="border-radius: 0%;height:20px; width: 20px; overflow: hidden;"  src="https://static.fansland.io/email/medium.png" alt="">
              </a>
            </p>
            <p style="color: #807E7E; font-size: 12px; text-align: center;color:black;">
              ©2024 Fansland. All rights reserved
            </p>
        </div>
    </div>
</div>
    """

    return first_part