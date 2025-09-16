# 🎨 SendGrid Dynamic Template Setup for AegisOne

This guide walks you through creating a professional dynamic email template in SendGrid for your AegisOne welcome emails.

## 🚀 Quick Setup Steps

### Step 1: Create Dynamic Template in SendGrid

1. **Login to SendGrid Dashboard**
   ```
   https://app.sendgrid.com/
   ```

2. **Navigate to Dynamic Templates**
   - Go to **Email API** → **Dynamic Templates**
   - Click **"Create a Dynamic Template"**

3. **Create Template**
   - **Template Name**: `AegisOne Welcome Email`
   - Click **"Create"**
   - **Copy the Template ID** (e.g., `d-1234567890abcdef`) - you'll need this!

### Step 2: Add Template Version

1. **Click "Add Version"**
2. **Choose "Code Editor"** (not Design Editor)
3. **Template Settings**:
   - **Version Name**: `v1.0 - Welcome Email`
   - **Subject**: `🛡️ Welcome to AegisOne - Your AI Security Agent`

4. **Paste Template Code**:
   - Copy the entire contents of `sendgrid-template.html`
   - Paste into the **HTML** section
   - SendGrid will auto-generate the plain text version

5. **Test Data** (for preview):
   ```json
   {
     "firstName": "John",
     "platform": "Windows", 
     "websiteUrl": "https://aegisone.com",
     "unsubscribeUrl": "{{unsubscribe}}"
   }
   ```

6. **Preview & Save**
   - Use the preview to see how it looks
   - Click **"Save"** when satisfied

### Step 3: Configure Firebase Functions

Add the template ID to your Firebase Functions config:

```bash
# Set the template ID you copied from SendGrid
firebase functions:config:set sendgrid.template_id="d-your-template-id-here"

# Optional: Set custom sender details
firebase functions:config:set email.from="noreply@aegisone.com"
firebase functions:config:set email.name="AegisOne"
firebase functions:config:set website.url="https://aegisone.com"
```

### Step 4: Deploy Updated Functions

```bash
firebase deploy --only functions
```

## 📧 Template Variables

Your template uses these dynamic variables:

| Variable | Type | Description | Example |
|----------|------|-------------|---------|
| `{{firstName}}` | string | User's first name from email | "John" |
| `{{platform}}` | string | Selected platform | "Windows" |
| `{{websiteUrl}}` | string | Your website URL | "https://aegisone.com" |
| `{{unsubscribeUrl}}` | string | Auto-generated unsubscribe link | SendGrid managed |

## 🎨 Template Features

### ✅ **Professional Design**
- **Mobile-responsive** design
- **Dark theme** matching your brand
- **AegisOne colors** (cyber blue/purple)
- **Email client compatibility** (Outlook, Gmail, Apple Mail, etc.)

### ✅ **Dynamic Content**
- **Personalized greeting** with first name
- **Platform-specific messaging**
- **Branded call-to-action button**
- **Professional footer** with unsubscribe

### ✅ **Email Best Practices**
- **HTML + Plain Text** versions
- **Unsubscribe compliance**
- **Mobile optimization**
- **Dark mode support**

## 🧪 Testing Your Template

### Preview in SendGrid
1. Go to your template in SendGrid
2. Click **"Preview"**
3. Use test data to see different scenarios:
   ```json
   {
     "firstName": "Sarah",
     "platform": "macOS",
     "websiteUrl": "https://aegisone.com"
   }
   ```

### Test Live Email
```bash
# Send test email through your function
curl -X POST https://your-region-aegonisone.cloudfunctions.net/resend_welcome_email \
  -H "Content-Type: application/json" \
  -d '{"email": "your-test-email@example.com"}'
```

## 📊 Email Analytics

SendGrid provides detailed analytics for your template:

### **Delivery Metrics**
- **Delivered**: Successfully delivered emails
- **Bounces**: Invalid/non-existent email addresses
- **Blocks**: Emails blocked by recipient's server

### **Engagement Metrics**  
- **Opens**: How many recipients opened the email
- **Clicks**: Clicks on your "Visit AegisOne" button
- **Unsubscribes**: Users who opted out

### **View Analytics**
1. Go to **Analytics** → **Email Activity**
2. Filter by your template ID
3. View real-time delivery and engagement data

## 🔧 Customization Options

### Change Email Subject
```bash
# Update the subject line in SendGrid template settings
# Or make it dynamic:
firebase functions:config:set email.subject="🛡️ Welcome {{firstName}} to AegisOne"
```

### Add More Dynamic Variables
1. **Edit template HTML** to include new variables like `{{lastName}}`
2. **Update Cloud Function** to pass additional data
3. **Redeploy** functions with new template data

### A/B Testing
1. **Create multiple template versions** in SendGrid
2. **Randomize template selection** in your Cloud Function
3. **Compare metrics** to optimize engagement

## 🚨 Troubleshooting

### **"Template not found" Error**
```bash
# Check your template ID
firebase functions:config:get

# Update if incorrect
firebase functions:config:set sendgrid.template_id="d-correct-id-here"
firebase deploy --only functions
```

### **Variables Not Rendering**
- Ensure variables in HTML use `{{variableName}}` format (double curly braces)
- Check that variable names match between template and Cloud Function
- Variables are case-sensitive

### **Template Not Mobile-Friendly**
- The provided template is already responsive
- Test on multiple devices using SendGrid's preview
- Ensure images and buttons are appropriately sized

### **High Bounce Rate**
- Verify sender domain authentication in SendGrid
- Use a professional from-address (not gmail/yahoo)
- Check email addresses for typos before sending

## 💡 Pro Tips

### **Sender Authentication**
1. **Domain Authentication**: Set up DNS records for your domain
2. **Link Branding**: Brand your click tracking links
3. **Reputation**: Start with low volume, gradually increase

### **Email Deliverability**
- **Clean list**: Remove invalid emails promptly
- **Engagement**: Higher opens/clicks = better inbox placement
- **Content**: Avoid spam trigger words

### **Performance Optimization**
- **Template caching**: SendGrid caches templates for fast delivery
- **Batch sending**: Send multiple emails efficiently
- **Error handling**: Graceful fallbacks if template fails

## 📈 Next Steps

1. **Monitor analytics** for the first week
2. **A/B test subject lines** to improve open rates
3. **Create additional templates** for different email types:
   - Product launch announcements
   - Feature update notifications
   - Security tips and best practices

---

**Template ID**: Remember to save your template ID from SendGrid and add it to Firebase config!

**Support**: Check the SendGrid documentation for advanced features and troubleshooting.
