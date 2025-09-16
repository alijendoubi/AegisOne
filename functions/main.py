"""
AegisOne Waitlist Email Automation
Automatically sends welcome emails when users join the waitlist
"""

import os
import json
from datetime import datetime
from typing import Dict, Any

from firebase_functions import firestore_fn, https_fn
from firebase_functions.options import set_global_options
from firebase_admin import initialize_app, firestore
from sendgrid import SendGridAPIClient
from sendgrid.helpers.mail import Mail, Email, To, Content, CustomArg
from jinja2 import Template

# Initialize Firebase
initialize_app()
set_global_options(max_instances=10)

# Email templates
WELCOME_EMAIL_TEMPLATE = """
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Welcome to AegisOne</title>
    <style>
        body { font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif; line-height: 1.6; color: #e6eef7; background: #0b0f17; margin: 0; padding: 20px; }
        .container { max-width: 600px; margin: 0 auto; background: linear-gradient(180deg, rgba(255,255,255,.06), rgba(255,255,255,.02)); border: 1px solid rgba(255,255,255,.08); border-radius: 16px; overflow: hidden; }
        .header { padding: 40px 40px 20px; text-align: center; background: linear-gradient(135deg, rgba(0,230,255,.15), rgba(124,58,237,.15)); }
        .logo { width: 48px; height: 48px; margin: 0 auto 16px; border-radius: 12px; background: radial-gradient(100% 100% at 30% 30%, #00e6ff 0%, #7c3aed 60%, #0b0f17 100%); }
        .content { padding: 30px 40px 40px; }
        h1 { color: #00e6ff; font-size: 28px; margin-bottom: 16px; font-weight: 700; }
        .badge { display: inline-block; padding: 6px 12px; background: rgba(0,230,255,.15); color: #00e6ff; border-radius: 20px; font-size: 12px; font-weight: 600; margin-bottom: 24px; }
        .feature { margin: 20px 0; padding: 16px; background: rgba(255,255,255,.02); border: 1px solid rgba(255,255,255,.04); border-radius: 8px; }
        .feature-icon { display: inline-block; margin-right: 12px; }
        .cta { margin: 30px 0; text-align: center; }
        .btn { display: inline-block; padding: 14px 28px; background: linear-gradient(135deg, #00e6ff 0%, #7c3aed 100%); color: white; text-decoration: none; border-radius: 8px; font-weight: 600; }
        .footer { padding: 20px 40px; border-top: 1px solid rgba(255,255,255,.08); font-size: 14px; color: #9fb3c8; text-align: center; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="logo"></div>
            <h1>Welcome to AegisOne</h1>
            <div class="badge">AI Security Agent</div>
        </div>
        
        <div class="content">
            <p>Hi {{ name or 'there' }},</p>
            
            <p>Thanks for joining the <strong>AegisOne waitlist</strong>! You're now part of the next generation of cybersecurity.</p>
            
            <div class="feature">
                <span class="feature-icon">🛡️</span>
                <strong>Kill-Switch Speed:</strong> Stop ransomware process trees in seconds
            </div>
            
            <div class="feature">
                <span class="feature-icon">♻️</span>
                <strong>Snapshot Rollback:</strong> Restore files instantly from safe snapshots
            </div>
            
            <div class="feature">
                <span class="feature-icon">🧠</span>
                <strong>Explainable AI:</strong> Get plain-language incident reports
            </div>
            
            <div class="feature">
                <span class="feature-icon">🔒</span>
                <strong>Privacy-First:</strong> Everything runs on your device
            </div>
            
            <p><strong>What happens next?</strong></p>
            <ul>
                <li>We'll keep you updated on our progress</li>
                <li>You'll get early access when {{ platform }} support is ready</li>
                <li>Be first to know about beta releases and security updates</li>
            </ul>
            
            <div class="cta">
                <a href="https://aegisone.com" class="btn">Visit AegisOne</a>
            </div>
            
            <p><em>AegisOne is defensive-only. We never hack back.</em></p>
        </div>
        
        <div class="footer">
            <p>AegisOne &bull; AI Security Agent<br>
            You can unsubscribe from these emails at any time.</p>
        </div>
    </div>
</body>
</html>
"""

TEXT_EMAIL_TEMPLATE = """
Welcome to AegisOne!

Hi {{ name or 'there' }},

Thanks for joining the AegisOne waitlist! You're now part of the next generation of cybersecurity.

AegisOne Features:
🛡️ Kill-Switch Speed: Stop ransomware process trees in seconds
♻️ Snapshot Rollback: Restore files instantly from safe snapshots  
🧠 Explainable AI: Get plain-language incident reports
🔒 Privacy-First: Everything runs on your device

What happens next?
- We'll keep you updated on our progress
- You'll get early access when {{ platform }} support is ready
- Be first to know about beta releases and security updates

Visit us: https://aegisone.com

AegisOne is defensive-only. We never hack back.

AegisOne Team
"""

def send_welcome_email(email_address: str, user_data: Dict[str, Any]) -> bool:
    """
    Send welcome email using SendGrid Dynamic Template
    """
    try:
        # Get configuration from environment
        sendgrid_api_key = os.environ.get('SENDGRID_API_KEY')
        template_id = os.environ.get('SENDGRID_TEMPLATE_ID')  # Add this to your config
        from_email = os.environ.get('FROM_EMAIL', 'noreply@aegisone.com')
        from_name = os.environ.get('FROM_NAME', 'AegisOne')
        website_url = os.environ.get('WEBSITE_URL', 'https://aegisone.com')
        
        if not sendgrid_api_key:
            print("❌ SENDGRID_API_KEY not found in environment")
            return False
            
        if not template_id:
            print("❌ SENDGRID_TEMPLATE_ID not found in environment")
            return False
            
        # Extract first name from email
        first_name = user_data.get('email', '').split('@')[0].capitalize()
        if not first_name or len(first_name) < 2:
            first_name = 'there'
            
        # Prepare dynamic template data
        template_data = {
            'firstName': first_name,
            'platform': user_data.get('platform', 'your platform'),
            'websiteUrl': website_url,
            'unsubscribeUrl': '{{unsubscribe}}' # SendGrid will replace this
        }
        
        # Create email using dynamic template
        message = Mail(
            from_email=Email(from_email, from_name),
            to_emails=To(email_address)
        )
        
        # Set dynamic template ID and data
        message.template_id = template_id
        message.dynamic_template_data = template_data
        
        # Optional: Add custom headers using the correct method
        message.add_custom_arg(CustomArg('source', 'waitlist_welcome'))
        message.add_custom_arg(CustomArg('platform', user_data.get('platform', 'unknown')))
        message.add_custom_arg(CustomArg('signup_date', datetime.utcnow().isoformat()))
        
        # Send email
        sg = SendGridAPIClient(api_key=sendgrid_api_key)
        response = sg.send(message)
        
        if response.status_code in [200, 201, 202]:
            print(f"✅ Welcome email sent to {email_address} using template {template_id}")
            return True
        else:
            print(f"❌ Email send failed: {response.status_code} - {response.body}")
            return False
            
    except Exception as e:
        print(f"❌ Error sending email to {email_address}: {str(e)}")
        return False

@firestore_fn.on_document_created(document="waitlist/{docId}")
def send_waitlist_welcome_email(event: firestore_fn.Event[firestore_fn.DocumentSnapshot]) -> None:
    """
    Triggered when a new document is created in the waitlist collection
    Automatically sends a welcome email to the new subscriber
    """
    try:
        # Get the document data
        doc_data = event.data.to_dict()
        if not doc_data:
            print("❌ No document data found")
            return
            
        email_address = doc_data.get('email')
        if not email_address:
            print("❌ No email address found in document")
            return
            
        print(f"📧 Processing welcome email for: {email_address}")
        
        # Send welcome email
        email_sent = send_welcome_email(email_address, doc_data)
        
        # Update the document with email status
        doc_ref = event.data.reference
        doc_ref.update({
            'welcome_email_sent': email_sent,
            'welcome_email_sent_at': datetime.utcnow(),
            'email_status': 'sent' if email_sent else 'failed'
        })
        
        print(f"✅ Welcome email processing completed for {email_address}")
        
    except Exception as e:
        print(f"❌ Error in welcome email function: {str(e)}")
        
        # Still try to update the document with error status
        try:
            doc_ref = event.data.reference
            doc_ref.update({
                'welcome_email_sent': False,
                'welcome_email_error': str(e),
                'email_status': 'error'
            })
        except:
            pass

@https_fn.on_request()
def resend_welcome_email(req: https_fn.Request) -> https_fn.Response:
    """
    Manual endpoint to resend welcome emails
    POST /resend_welcome_email with { "email": "user@example.com" }
    """
    try:
        if req.method != 'POST':
            return https_fn.Response("Method not allowed", status=405)
            
        data = req.get_json()
        if not data or 'email' not in data:
            return https_fn.Response("Email address required", status=400)
            
        email_address = data['email']
        
        # Find the user in Firestore
        db = firestore.client()
        waitlist_ref = db.collection('waitlist')
        query = waitlist_ref.where('email', '==', email_address).limit(1)
        docs = query.stream()
        
        user_doc = None
        for doc in docs:
            user_doc = doc
            break
            
        if not user_doc:
            return https_fn.Response("Email not found in waitlist", status=404)
            
        # Send welcome email
        user_data = user_doc.to_dict()
        email_sent = send_welcome_email(email_address, user_data)
        
        # Update document
        user_doc.reference.update({
            'welcome_email_sent': email_sent,
            'welcome_email_sent_at': datetime.utcnow(),
            'email_status': 'resent' if email_sent else 'failed',
            'resend_count': user_data.get('resend_count', 0) + 1
        })
        
        if email_sent:
            return https_fn.Response(json.dumps({
                'success': True,
                'message': f'Welcome email resent to {email_address}'
            }), content_type='application/json')
        else:
            return https_fn.Response(json.dumps({
                'success': False,
                'message': 'Failed to send email'
            }), status=500, content_type='application/json')
            
    except Exception as e:
        return https_fn.Response(json.dumps({
            'success': False,
            'message': f'Error: {str(e)}'
        }), status=500, content_type='application/json')
