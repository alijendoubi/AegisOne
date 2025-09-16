# 🚀 AegisOne Fresh SendGrid Setup Guide

This guide will help you set up a completely fresh SendGrid configuration with a new API key and template.

## Prerequisites

- SendGrid account
- Firebase CLI installed and authenticated
- Python 3.13 with virtual environment support

## Quick Start

Run the automated setup script:

```bash
./setup_sendgrid_fresh.sh
```

## Step-by-Step Manual Process

If you prefer to do it manually, follow these steps:

### 1. Create New SendGrid API Key

1. Go to [SendGrid API Keys](https://app.sendgrid.com/settings/api_keys)
2. Click **"Create API Key"**
3. Choose **"Restricted Access"** for better security
4. Set these permissions:
   - Mail Send: **FULL ACCESS**
   - Template Engine: **READ ACCESS** (for templates)
5. Name it: `AegisOne-Production-YYYYMMDD`
6. **Copy the key immediately** (you won't see it again!)

### 2. Create Dynamic Template

1. Go to [Dynamic Templates](https://app.sendgrid.com/dynamic_templates)
2. Click **"Create a Dynamic Template"**
3. Name: `AegisOne Welcome Email`
4. Click on the template to edit
5. Click **"Add Version"**
6. Choose **"Code Editor"**
7. Copy the entire content from `sendgrid-template.html` in this directory
8. Click **"Save"**
9. Copy the **Template ID** (starts with `d-`)

### 3. Update Environment Configuration

Create/update `functions/.env`:

```env
SENDGRID_API_KEY=your_new_api_key_here
SENDGRID_TEMPLATE_ID=d-your-template-id-here
FROM_EMAIL=your-sender@domain.com
FROM_NAME=AegisOne
WEBSITE_URL=https://aegisone.com
NODE_ENV=production
```

### 4. Deploy Functions

```bash
firebase deploy --only functions
```

### 5. Test the Setup

```bash
curl -X POST https://us-central1-aegonisone.cloudfunctions.net/resend_welcome_email \
  -H "Content-Type: application/json" \
  -d '{"email": "your-test@email.com"}'
```

## Expected Response

✅ **Success Response:**
```json
{"success": true, "message": "Welcome email resent to your-test@email.com"}
```

❌ **Error Response:**
```json
{"success": false, "message": "Failed to send email"}
```

## Troubleshooting

### Common Issues

1. **401 Unauthorized**
   - Check API key is valid
   - Verify API key has Mail Send permissions

2. **Template not found**
   - Verify Template ID is correct
   - Check template is active in SendGrid

3. **Function deployment fails**
   - Ensure virtual environment exists: `python3.13 -m venv functions/venv`
   - Install dependencies: `pip install -r functions/requirements.txt`

### Debug Commands

```bash
# Check function logs
firebase functions:log

# Test API key manually
curl -X GET "https://api.sendgrid.com/v3/user/profile" \
  -H "Authorization: Bearer YOUR_API_KEY"

# Check function status
firebase functions:list
```

## Template Variables

The dynamic template uses these variables:

- `{{firstName}}` - Extracted from email address
- `{{platform}}` - User's platform preference
- `{{websiteUrl}}` - Your website URL
- `{{unsubscribeUrl}}` - SendGrid unsubscribe link

## Security Best Practices

1. **Never commit `.env` files** (already in `.gitignore`)
2. **Use restricted API keys** with minimal permissions
3. **Rotate API keys** regularly
4. **Monitor SendGrid activity** for suspicious usage
5. **Set up billing alerts** in SendGrid

## Function URLs

After deployment, your functions will be available at:

- **Automatic trigger:** `https://us-central1-aegonisone.cloudfunctions.net/send_waitlist_welcome_email`
- **Manual resend:** `https://us-central1-aegonisone.cloudfunctions.net/resend_welcome_email`

## Next Steps

1. **Test waitlist signup** on your website
2. **Monitor email delivery** in SendGrid Activity
3. **Set up webhooks** for delivery tracking (optional)
4. **Configure domain authentication** for better deliverability
5. **Set up dedicated IP** for high-volume sending (if needed)

## Support

- **SendGrid Docs:** https://docs.sendgrid.com/
- **Firebase Functions:** https://firebase.google.com/docs/functions
- **Template Testing:** Use SendGrid's template testing feature

---

🎉 **Your AegisOne email system will be ready to send welcome emails automatically!**
