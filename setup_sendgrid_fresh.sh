#!/bin/bash

# 🚀 AegisOne SendGrid Fresh Setup
# Complete setup with new API key generation

set -e

echo "🔄 AegisOne SendGrid Fresh Setup"
echo "================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_step() {
    echo -e "${BLUE}📋 $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Step 1: Clean up old configuration
print_step "Cleaning up old configuration..."

# Remove old Firebase config
firebase functions:config:unset sendgrid email 2>/dev/null || true
print_success "Cleared old Firebase config"

# Step 2: Generate new SendGrid API key
print_step "Setting up new SendGrid API key..."
echo ""
echo "🔗 Please follow these steps to create a new SendGrid API key:"
echo ""
echo "1. Go to: https://app.sendgrid.com/settings/api_keys"
echo "2. Click 'Create API Key'"
echo "3. Choose 'Full Access' or 'Restricted Access' with Mail Send permissions"
echo "4. Name it: 'AegisOne-$(date +%Y%m%d)'"
echo "5. Copy the API key (it starts with 'SG.')"
echo ""
print_warning "The API key will only be shown once, so copy it carefully!"
echo ""

# Get new API key from user
while true; do
    read -s -p "📝 Enter your new SendGrid API Key: " SENDGRID_API_KEY
    echo ""
    
    if [[ -z "$SENDGRID_API_KEY" ]]; then
        print_error "API key cannot be empty!"
        continue
    fi
    
    if [[ ! "$SENDGRID_API_KEY" =~ ^SG\. ]]; then
        print_error "Invalid API key format. It should start with 'SG.'"
        continue
    fi
    
    # Test the API key
    print_step "Testing API key..."
    
    HTTP_CODE=$(curl -s -o /tmp/sendgrid_test.json -w "%{http_code}" \
        -X GET "https://api.sendgrid.com/v3/user/profile" \
        -H "Authorization: Bearer $SENDGRID_API_KEY")
    
    if [ "$HTTP_CODE" = "200" ]; then
        print_success "API key is valid!"
        break
    else
        print_error "API key test failed (HTTP $HTTP_CODE). Please check your key."
        echo "Response: $(cat /tmp/sendgrid_test.json 2>/dev/null || echo 'No response')"
        continue
    fi
done

# Step 3: Set up dynamic template
print_step "Setting up SendGrid Dynamic Template..."
echo ""
echo "🔗 Please follow these steps to create a dynamic template:"
echo ""
echo "1. Go to: https://app.sendgrid.com/dynamic_templates"
echo "2. Click 'Create a Dynamic Template'"
echo "3. Name it: 'AegisOne Welcome Email'"
echo "4. Click on the template to edit it"
echo "5. Click 'Add Version'"
echo "6. Choose 'Code Editor'"
echo "7. Copy the template from 'sendgrid-template.html' in this directory"
echo "8. Save and get the Template ID (starts with 'd-')"
echo ""

read -p "📝 Enter your SendGrid Template ID: " TEMPLATE_ID

if [[ -z "$TEMPLATE_ID" ]]; then
    print_error "Template ID is required!"
    exit 1
fi

if [[ ! "$TEMPLATE_ID" =~ ^d- ]]; then
    print_error "Invalid template ID format. It should start with 'd-'"
    exit 1
fi

# Step 4: Configure sender email
print_step "Configuring sender email..."
echo ""
read -p "📧 Enter sender email (press Enter for default: noreply@aegisone.com): " FROM_EMAIL
FROM_EMAIL=${FROM_EMAIL:-noreply@aegisone.com}

read -p "👤 Enter sender name (press Enter for default: AegisOne): " FROM_NAME
FROM_NAME=${FROM_NAME:-AegisOne}

read -p "🌐 Enter website URL (press Enter for default: https://aegisone.com): " WEBSITE_URL
WEBSITE_URL=${WEBSITE_URL:-https://aegisone.com}

# Step 5: Create new .env file
print_step "Creating new environment configuration..."

cat > functions/.env << EOF
# SendGrid Configuration (Generated $(date))
SENDGRID_API_KEY=$SENDGRID_API_KEY
SENDGRID_TEMPLATE_ID=$TEMPLATE_ID
FROM_EMAIL=$FROM_EMAIL
FROM_NAME=$FROM_NAME
WEBSITE_URL=$WEBSITE_URL

# Additional Settings
NODE_ENV=production
EOF

print_success "Environment configuration created"

# Step 6: Install dependencies if needed
print_step "Checking Python dependencies..."

if [ ! -d "functions/venv" ]; then
    print_step "Creating Python virtual environment..."
    cd functions
    python3.13 -m venv venv
    cd ..
    print_success "Virtual environment created"
fi

print_step "Installing/updating dependencies..."
cd functions
source venv/bin/activate
pip install -r requirements.txt --upgrade
cd ..
print_success "Dependencies updated"

# Step 7: Deploy functions
print_step "Deploying Firebase Functions..."
firebase deploy --only functions

if [ $? -eq 0 ]; then
    print_success "Functions deployed successfully!"
else
    print_error "Function deployment failed!"
    exit 1
fi

# Step 8: Test the setup
print_step "Testing email functionality..."
echo ""

read -p "📧 Enter email address to test with: " TEST_EMAIL

if [[ -n "$TEST_EMAIL" ]]; then
    print_step "Sending test email..."
    
    FUNCTION_URL="https://us-central1-aegonisone.cloudfunctions.net/resend_welcome_email"
    
    RESPONSE=$(curl -s -X POST "$FUNCTION_URL" \
        -H "Content-Type: application/json" \
        -d "{\"email\": \"$TEST_EMAIL\"}")
    
    if echo "$RESPONSE" | grep -q '"success": true'; then
        print_success "Test email sent successfully!"
        print_success "Check $TEST_EMAIL for the welcome email"
    else
        print_error "Test email failed:"
        echo "$RESPONSE"
    fi
fi

# Step 9: Display final configuration
echo ""
echo "🎉 Setup Complete!"
echo "==================="
echo ""
print_success "SendGrid API Key: Configured (hidden for security)"
print_success "Template ID: $TEMPLATE_ID"
print_success "From Email: $FROM_EMAIL"
print_success "From Name: $FROM_NAME"
print_success "Website URL: $WEBSITE_URL"
echo ""
print_success "Functions deployed at:"
echo "  • Automatic: https://us-central1-aegonisone.cloudfunctions.net/send_waitlist_welcome_email"
echo "  • Manual: https://us-central1-aegonisone.cloudfunctions.net/resend_welcome_email"
echo ""

# Step 10: Security reminder
print_warning "Security Reminders:"
echo "  • Keep your SendGrid API key secure"
echo "  • The .env file is in .gitignore to prevent committing secrets"
echo "  • Monitor your SendGrid usage and billing"
echo "  • Consider setting up SendGrid webhooks for delivery tracking"
echo ""

# Step 11: Next steps
print_step "Next Steps:"
echo "  1. Test the waitlist signup on your website"
echo "  2. Monitor Firebase Functions logs: firebase functions:log"
echo "  3. Check SendGrid activity dashboard for email statistics"
echo "  4. Set up email templates for other notifications if needed"
echo ""

print_success "AegisOne email system is now ready! 🚀"

# Clean up temporary files
rm -f /tmp/sendgrid_test.json 2>/dev/null || true
