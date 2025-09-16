# 📧 AegisOne Email Automation Setup

This guide will help you set up automated welcome emails for your AegisOne waitlist.

## 🚀 Quick Setup

### 1. SendGrid Account Setup

1. **Sign up for SendGrid** (free tier includes 100 emails/day):
   ```
   https://signup.sendgrid.com/
   ```

2. **Create API Key**:
   - Go to Settings → API Keys
   - Click "Create API Key"
   - Choose "Restricted Access"
   - Give permissions: `Mail Send` (Full Access)
   - Name it: `AegisOne Waitlist Emails`
   - **Save the API key** - you'll need it next!

3. **Verify Sender Identity**:
   - Go to Settings → Sender Authentication
   - Choose "Single Sender Verification" (easiest)
   - Add: `noreply@aegisone.com` or your domain
   - Verify the email address

### 2. Configure Firebase Functions

Set the SendGrid API key as an environment variable:

```bash
firebase functions:config:set sendgrid.api_key="YOUR_SENDGRID_API_KEY_HERE"
```

### 3. Deploy the Functions

```bash
# Deploy the email automation functions
firebase deploy --only functions
```

## 📋 What This Sets Up

### ✅ **Automatic Welcome Emails**
- **Trigger**: When someone joins the waitlist (Firestore document created)
- **Content**: Beautiful HTML email with AegisOne branding
- **Features**: Platform-specific messaging, plain-text fallback

### ✅ **Email Tracking**
- **Status Updates**: Each waitlist entry gets email status fields
- **Retry Logic**: Manual resend endpoint for failed emails
- **Error Logging**: Detailed logs for troubleshooting

### ✅ **Manual Resend Endpoint**
```bash
# Resend welcome email to specific user
curl -X POST https://your-region-aegonisone.cloudfunctions.net/resend_welcome_email \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com"}'
```

## 🎨 Email Template Features

The welcome email includes:
- **AegisOne branding** with your color scheme
- **Feature highlights** (Kill-Switch, Rollback, AI, Privacy)
- **Platform-specific messaging** (Windows/macOS/Linux)
- **Clear next steps** for subscribers
- **Responsive design** for mobile devices

## 🔧 Configuration Options

### Environment Variables
```bash
# Required
SENDGRID_API_KEY=your_api_key_here

# Optional (defaults shown)
FROM_EMAIL=noreply@aegisone.com
FROM_NAME=AegisOne
WEBSITE_URL=https://aegisone.com
```

### Customizing Email Templates
Edit the templates in `functions/main.py`:
- `WELCOME_EMAIL_TEMPLATE` - HTML version
- `TEXT_EMAIL_TEMPLATE` - Plain text version

## 📊 Monitoring & Analytics

### Check Email Status
```bash
# View function logs
firebase functions:log

# Check Firestore for email status
# Each waitlist document will have:
# - welcome_email_sent: boolean
# - welcome_email_sent_at: timestamp  
# - email_status: "sent" | "failed" | "error"
```

### SendGrid Dashboard
Monitor email delivery, opens, clicks, and bounces in your SendGrid dashboard.

## 🚨 Troubleshooting

### Common Issues

**1. "SENDGRID_API_KEY not found"**
```bash
firebase functions:config:get
firebase functions:config:set sendgrid.api_key="your_key"
firebase deploy --only functions
```

**2. "Email send failed: 401"**
- Check API key permissions in SendGrid
- Ensure "Mail Send" permission is enabled

**3. "From email not verified"**
- Complete sender verification in SendGrid
- Use verified email in FROM_EMAIL

**4. Function timeout**
```bash
# Increase timeout in functions/main.py
set_global_options(max_instances=10, timeout_sec=60)
```

## 💰 Cost Estimates

### SendGrid Free Tier
- **100 emails/day** free
- **40,000 emails/month** for first 30 days
- Perfect for early waitlist growth

### Firebase Functions
- **2 million invocations/month** free
- Each email = 1 invocation
- Essentially free for typical usage

## 🔒 Security & Privacy

- **No email content logging** (only success/failure)
- **Secure API key storage** via Firebase environment
- **Unsubscribe links** included in emails
- **GDPR compliant** data handling

## 📈 Scaling Up

When you outgrow the free tier:
- **SendGrid Pro**: $14.95/month for 40K emails
- **Alternative providers**: AWS SES, Mailgun, Postmark
- **Email marketing platforms**: ConvertKit, Mailchimp (for campaigns)

---

**Next Steps**: 
1. Set up SendGrid account
2. Configure API key
3. Deploy functions
4. Test with a real email signup!
