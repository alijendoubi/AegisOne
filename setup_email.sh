#!/bin/bash

# 📧 AegisOne Email Automation Setup Script
# This script helps you configure SendGrid for automated welcome emails

echo "🛡️  AegisOne Email Automation Setup"
echo "===================================="
echo ""

# Check if Firebase CLI is available
if ! command -v firebase &> /dev/null; then
    echo "❌ Firebase CLI not found. Please install it first:"
    echo "   npm install -g firebase-tools"
    exit 1
fi

# Check if we're in a Firebase project
if [ ! -f "firebase.json" ]; then
    echo "❌ firebase.json not found. Please run this from your Firebase project root."
    exit 1
fi

echo "✅ Firebase CLI found"
echo "✅ Firebase project detected"
echo ""

# Get SendGrid API Key
echo "📝 Please enter your SendGrid API Key:"
echo "   (Get it from: https://app.sendgrid.com/settings/api_keys)"
echo ""
read -p "SendGrid API Key: " -s SENDGRID_API_KEY
echo ""

if [ -z "$SENDGRID_API_KEY" ]; then
    echo "❌ SendGrid API Key is required"
    exit 1
fi

# Set the API key in Firebase Functions config
echo "🔧 Setting SendGrid API key in Firebase Functions..."
firebase functions:config:set sendgrid.api_key="$SENDGRID_API_KEY"

if [ $? -eq 0 ]; then
    echo "✅ SendGrid API key configured successfully"
else
    echo "❌ Failed to set SendGrid API key"
    exit 1
fi

echo ""

# Get SendGrid Template ID
echo "📝 SendGrid Dynamic Template Setup:"
echo "   1. Go to: https://app.sendgrid.com/"
echo "   2. Navigate to Email API → Dynamic Templates"
echo "   3. Create template using sendgrid-template.html"
echo "   4. Copy the Template ID (starts with 'd-')"
echo ""
read -p "📧 Enter SendGrid Template ID (d-xxxxx): " TEMPLATE_ID

if [ ! -z "$TEMPLATE_ID" ]; then
    firebase functions:config:set sendgrid.template_id="$TEMPLATE_ID"
    echo "✅ Template ID set to: $TEMPLATE_ID"
else
    echo "⚠️  No template ID provided - using basic email template"
fi

echo ""

# Optional: Set custom sender email
read -p "📧 Enter sender email (default: noreply@aegisone.com): " FROM_EMAIL
if [ ! -z "$FROM_EMAIL" ]; then
    firebase functions:config:set email.from="$FROM_EMAIL"
    echo "✅ Sender email set to: $FROM_EMAIL"
fi

# Optional: Set website URL
read -p "🌐 Enter website URL (default: https://aegisone.com): " WEBSITE_URL
if [ ! -z "$WEBSITE_URL" ]; then
    firebase functions:config:set website.url="$WEBSITE_URL"
    echo "✅ Website URL set to: $WEBSITE_URL"
fi

echo ""

# Deploy functions
echo "🚀 Deploying Cloud Functions..."
echo "   This may take a few minutes..."
echo ""

firebase deploy --only functions

if [ $? -eq 0 ]; then
    echo ""
    echo "🎉 Email automation setup complete!"
    echo ""
    echo "📋 What's been configured:"
    echo "   ✅ SendGrid API key stored securely"
    if [ ! -z "$TEMPLATE_ID" ]; then
        echo "   ✅ Dynamic email template: $TEMPLATE_ID"
    fi
    echo "   ✅ Welcome email Cloud Function deployed"
    echo "   ✅ Auto-trigger on new waitlist signups"
    echo "   ✅ Manual resend endpoint available"
    echo ""
    echo "🧪 Test your setup:"
    echo "   1. Go to your landing page"
    echo "   2. Sign up with your email"
    echo "   3. Check your inbox for the welcome email"
    echo ""
    echo "📊 Monitor emails:"
    echo "   • Firebase Console: Functions logs"
    echo "   • SendGrid Dashboard: Delivery stats"
    echo "   • Firestore: Email status in waitlist documents"
    echo ""
    echo "📖 Documentation:"
    echo "   • EMAIL_SETUP.md - Basic setup"
    echo "   • SENDGRID_TEMPLATE_SETUP.md - Dynamic template guide"
    echo "   • sendgrid-template.html - Template code"
else
    echo ""
    echo "❌ Deployment failed!"
    echo "Check the error messages above and try again."
    echo ""
    echo "🔧 Common fixes:"
    echo "   • Make sure you're logged into Firebase: firebase login"
    echo "   • Check your Firebase project: firebase use --list"
    echo "   • Verify SendGrid API key has Mail Send permissions"
fi

echo ""
echo "Need help? Check EMAIL_SETUP.md for detailed instructions."
