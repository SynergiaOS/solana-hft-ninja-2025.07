import scrapy
import json
import re
from datetime import datetime, timedelta
from urllib.parse import urljoin, urlparse


class ProjectAuditorSpider(scrapy.Spider):
    name = "project_auditor"
    allowed_domains = ["github.com", "gitlab.com", "twitter.com", "t.me"]

    # Projects to monitor (can be loaded from external config)
    target_projects = [
        {
            'name': 'Example Memecoin',
            'website': 'https://example-memecoin.com',
            'github': 'https://github.com/example/memecoin',
            'twitter': 'https://twitter.com/examplememe',
            'telegram': 'https://t.me/examplememe',
            'contract_address': '7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU',
        }
        # Add more projects dynamically
    ]

    # Risk indicators to look for
    risk_indicators = [
        'contract updated', 'ownership transferred', 'liquidity removed',
        'website down', '404 error', 'domain expired', 'ssl expired',
        'repository deleted', 'commits removed', 'team left',
        'social media deleted', 'telegram closed', 'discord banned'
    ]

    custom_settings = {
        'DOWNLOAD_DELAY': 1,
        'RANDOMIZE_DOWNLOAD_DELAY': True,
        'USER_AGENT': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
        'COOKIES_ENABLED': False,
        'ROBOTSTXT_OBEY': True,
    }

    def start_requests(self):
        """Generate requests for all project components"""
        for project in self.target_projects:
            project_name = project['name']

            # Check website
            if project.get('website'):
                yield scrapy.Request(
                    url=project['website'],
                    callback=self.parse_website,
                    meta={'project': project, 'component': 'website'},
                    errback=self.handle_error
                )

            # Check GitHub repository
            if project.get('github'):
                yield scrapy.Request(
                    url=project['github'],
                    callback=self.parse_github,
                    meta={'project': project, 'component': 'github'},
                    errback=self.handle_error
                )

            # Check social media
            if project.get('twitter'):
                yield scrapy.Request(
                    url=project['twitter'],
                    callback=self.parse_social,
                    meta={'project': project, 'component': 'twitter'},
                    errback=self.handle_error
                )

    def parse_website(self, response):
        """Audit project website for red flags"""
        project = response.meta['project']

        audit_data = {
            'project_name': project['name'],
            'component': 'website',
            'url': response.url,
            'status_code': response.status,
            'timestamp': datetime.now().isoformat(),
            'issues': [],
            'health_score': 100,  # Start with perfect score
        }

        # Check for SSL certificate
        if not response.url.startswith('https://'):
            audit_data['issues'].append('No SSL certificate')
            audit_data['health_score'] -= 20

        # Check for basic website elements
        title = response.css('title::text').get()
        if not title or len(title.strip()) < 5:
            audit_data['issues'].append('Missing or poor title')
            audit_data['health_score'] -= 10

        # Look for social media links
        social_links = response.css('a[href*="twitter.com"], a[href*="telegram"], a[href*="discord"]').getall()
        if len(social_links) < 2:
            audit_data['issues'].append('Limited social media presence')
            audit_data['health_score'] -= 15

        # Check for whitepaper or documentation
        docs_links = response.css('a[href*="whitepaper"], a[href*="docs"], a[href*="documentation"]').getall()
        if not docs_links:
            audit_data['issues'].append('No whitepaper or documentation found')
            audit_data['health_score'] -= 25

        # Look for team information
        team_content = response.css('*:contains("team"), *:contains("founder"), *:contains("developer")').getall()
        if not team_content:
            audit_data['issues'].append('No team information visible')
            audit_data['health_score'] -= 20

        # Check for recent updates (look for dates)
        date_pattern = r'\b(202[3-5]|Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)\b'
        recent_dates = re.findall(date_pattern, response.text, re.IGNORECASE)
        if not recent_dates:
            audit_data['issues'].append('No recent updates visible')
            audit_data['health_score'] -= 10

        yield {
            'type': 'project_audit',
            'data': audit_data,
            'source': 'project_auditor',
            'collected_at': datetime.now().isoformat()
        }

    def parse_github(self, response):
        """Audit GitHub repository for development activity"""
        project = response.meta['project']

        audit_data = {
            'project_name': project['name'],
            'component': 'github',
            'url': response.url,
            'status_code': response.status,
            'timestamp': datetime.now().isoformat(),
            'issues': [],
            'health_score': 100,
        }

        # Check if repository exists
        if response.status == 404:
            audit_data['issues'].append('Repository not found or deleted')
            audit_data['health_score'] = 0
            yield {
                'type': 'project_audit',
                'data': audit_data,
                'source': 'project_auditor',
                'collected_at': datetime.now().isoformat()
            }
            return

        # Check for recent commits
        commit_dates = response.css('.commit-date, .commit-time, [datetime]::attr(datetime)').getall()
        if commit_dates:
            # Parse most recent commit date
            try:
                latest_commit = max(commit_dates)
                # Simple check if commit is recent (within 30 days)
                audit_data['latest_commit'] = latest_commit
            except:
                audit_data['issues'].append('Cannot parse commit dates')
                audit_data['health_score'] -= 15
        else:
            audit_data['issues'].append('No recent commits found')
            audit_data['health_score'] -= 30

        # Check for README
        readme_link = response.css('a[href*="README"]').get()
        if not readme_link:
            audit_data['issues'].append('No README file')
            audit_data['health_score'] -= 20

        # Check for license
        license_info = response.css('.license-info, a[href*="license"]').get()
        if not license_info:
            audit_data['issues'].append('No license specified')
            audit_data['health_score'] -= 10

        # Check number of contributors
        contributors = response.css('.contributor, .avatar').getall()
        if len(contributors) < 2:
            audit_data['issues'].append('Single contributor (centralization risk)')
            audit_data['health_score'] -= 25

        # Look for suspicious commit messages
        commit_messages = response.css('.commit-message').getall()
        suspicious_patterns = ['remove', 'delete', 'hide', 'backdoor', 'exploit']
        for message in commit_messages:
            if any(pattern in message.lower() for pattern in suspicious_patterns):
                audit_data['issues'].append('Suspicious commit messages detected')
                audit_data['health_score'] -= 40
                break

        yield {
            'type': 'project_audit',
            'data': audit_data,
            'source': 'project_auditor',
            'collected_at': datetime.now().isoformat()
        }

    def parse_social(self, response):
        """Audit social media presence"""
        project = response.meta['project']
        component = response.meta['component']

        audit_data = {
            'project_name': project['name'],
            'component': component,
            'url': response.url,
            'status_code': response.status,
            'timestamp': datetime.now().isoformat(),
            'issues': [],
            'health_score': 100,
        }

        # Check if social media account exists
        if response.status == 404:
            audit_data['issues'].append(f'{component.title()} account not found or deleted')
            audit_data['health_score'] = 0
        elif 'suspended' in response.text.lower() or 'banned' in response.text.lower():
            audit_data['issues'].append(f'{component.title()} account suspended or banned')
            audit_data['health_score'] = 10
        else:
            # Check for recent activity
            if component == 'twitter':
                tweets = response.css('[data-testid="tweet"]').getall()
                if len(tweets) < 5:
                    audit_data['issues'].append('Limited Twitter activity')
                    audit_data['health_score'] -= 20

        yield {
            'type': 'project_audit',
            'data': audit_data,
            'source': 'project_auditor',
            'collected_at': datetime.now().isoformat()
        }

    def handle_error(self, failure):
        """Handle request errors and timeouts"""
        project = failure.request.meta.get('project', {})
        component = failure.request.meta.get('component', 'unknown')

        error_data = {
            'project_name': project.get('name', 'unknown'),
            'component': component,
            'url': failure.request.url,
            'error': str(failure.value),
            'timestamp': datetime.now().isoformat(),
            'health_score': 0,
            'issues': [f'Failed to access {component}: {failure.value}']
        }

        yield {
            'type': 'project_audit',
            'data': error_data,
            'source': 'project_auditor',
            'collected_at': datetime.now().isoformat()
        }
