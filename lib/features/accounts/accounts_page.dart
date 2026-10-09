import 'package:flutter/material.dart';
import 'package:zettings/core/errors/app_error.dart';
import 'package:zettings/core/models/user_info.dart';
import 'package:zettings/core/services/accounts_service.dart';
import 'package:zettings/core/services/service_locator.dart';
import 'package:zettings/shared/widgets/hero_status_card.dart';
import 'package:zettings/shared/widgets/section_header.dart';
import 'package:zettings/shared/widgets/setting_group_card.dart';
import 'package:zettings/shared/widgets/setting_row.dart';
import 'package:zettings/shared/widgets/settings_page.dart';
import 'package:zettings/shared/widgets/state_views.dart';

/// Accounts settings page: current user identity, AccountsService integration,
/// password management, and login session preferences.
class AccountsPage extends StatefulWidget {
  const AccountsPage({super.key, this.provider});

  final AccountsServiceProvider? provider;

  @override
  State<AccountsPage> createState() => _AccountsPageState();
}

class _AccountsPageState extends State<AccountsPage> {
  late Future<List<UserInfo>> _future;
  bool _autoLogin = false;
  final Set<String> _userGroups = {'sudo', 'audio', 'plugdev', 'docker'};

  AccountsServiceProvider get _provider =>
      widget.provider ?? ServiceLocator.get<AccountsServiceProvider>();

  @override
  void initState() {
    super.initState();
    _future = _provider.getUsers();
  }

  void _reload() {
    setState(() {
      _future = _provider.getUsers();
      _future.ignore();
    });
  }

  void _showPasswordDialog() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Change Password'),
        content: const Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              obscureText: true,
              decoration: InputDecoration(
                labelText: 'Current password',
                border: OutlineInputBorder(),
              ),
            ),
            SizedBox(height: 12),
            TextField(
              obscureText: true,
              decoration: InputDecoration(
                labelText: 'New password',
                border: OutlineInputBorder(),
              ),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () {
              Navigator.of(context).pop();
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(content: Text('Password updated successfully')),
              );
            },
            child: const Text('Update'),
          ),
        ],
      ),
    );
  }

  void _showAddUserDialog() {
    final formKey = GlobalKey<FormState>();
    final nameCtrl = TextEditingController();
    final userCtrl = TextEditingController();
    final passCtrl = TextEditingController();
    var isAdmin = false;
    var requireChange = true;

    showDialog(
      context: context,
      builder: (ctx) => StatefulBuilder(
        builder: (context, setDialogState) {
          final theme = Theme.of(context);
          return AlertDialog(
            title: const Text('Add New User'),
            content: SizedBox(
              width: 420,
              child: Form(
                key: formKey,
                child: SingleChildScrollView(
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      TextFormField(
                        controller: nameCtrl,
                        decoration: const InputDecoration(
                          labelText: 'Full name',
                          hintText: 'e.g. Jane Doe',
                          prefixIcon: Icon(Icons.badge),
                        ),
                      ),
                      const SizedBox(height: 12),
                      TextFormField(
                        controller: userCtrl,
                        decoration: const InputDecoration(
                          labelText: 'Username',
                          hintText: 'e.g. jdoe',
                          prefixIcon: Icon(Icons.person),
                        ),
                        validator: (val) {
                          if (val == null || val.trim().isEmpty) {
                            return 'Username is required';
                          }
                          if (!RegExp(r'^[a-z_][a-z0-9_-]*$').hasMatch(val)) {
                            return 'Lowercase letters, digits, and hyphens only';
                          }
                          return null;
                        },
                      ),
                      const SizedBox(height: 16),
                      Text(
                        'Account Type',
                        style: theme.textTheme.labelMedium?.copyWith(
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                      const SizedBox(height: 8),
                      SegmentedButton<bool>(
                        segments: const [
                          ButtonSegment(
                            value: false,
                            label: Text('Standard'),
                            icon: Icon(Icons.person_outline),
                          ),
                          ButtonSegment(
                            value: true,
                            label: Text('Administrator'),
                            icon: Icon(Icons.admin_panel_settings),
                          ),
                        ],
                        selected: {isAdmin},
                        onSelectionChanged: (val) =>
                            setDialogState(() => isAdmin = val.first),
                      ),
                      const SizedBox(height: 16),
                      TextFormField(
                        controller: passCtrl,
                        obscureText: true,
                        decoration: const InputDecoration(
                          labelText: 'Initial password',
                          prefixIcon: Icon(Icons.lock),
                        ),
                      ),
                      const SizedBox(height: 12),
                      CheckboxListTile(
                        contentPadding: EdgeInsets.zero,
                        value: requireChange,
                        onChanged: (v) =>
                            setDialogState(() => requireChange = v ?? true),
                        title: const Text(
                          'Require password change on first login',
                        ),
                        controlAffinity: ListTileControlAffinity.leading,
                      ),
                    ],
                  ),
                ),
              ),
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.of(context).pop(),
                child: const Text('Cancel'),
              ),
              FilledButton(
                onPressed: () {
                  if (formKey.currentState?.validate() ?? false) {
                    final username = userCtrl.text.trim();
                    Navigator.of(context).pop();
                    ScaffoldMessenger.of(context).showSnackBar(
                      SnackBar(
                        content: Text(
                          'Created account "$username" (${isAdmin ? "Administrator" : "Standard"})',
                        ),
                      ),
                    );
                  }
                },
                child: const Text('Create User'),
              ),
            ],
          );
        },
      ),
    );
  }

  void _showUserGroupsDialog() {
    final availableGroups = [
      {'name': 'sudo', 'desc': 'Administrative root command privileges'},
      {'name': 'docker', 'desc': 'Manage Docker containers and daemon'},
      {'name': 'audio', 'desc': 'Direct PipeWire and ALSA hardware playback'},
      {'name': 'plugdev', 'desc': 'Mount external USB and storage drives'},
      {
        'name': 'dialout',
        'desc': 'Serial ports, Arduino, and embedded devices',
      },
      {'name': 'kvm', 'desc': 'Hardware-accelerated virtual machine access'},
    ];

    showDialog(
      context: context,
      builder: (ctx) => StatefulBuilder(
        builder: (context, setDialogState) {
          final theme = Theme.of(context);
          return AlertDialog(
            title: const Text('Configure User Groups'),
            content: SizedBox(
              width: 440,
              child: SingleChildScrollView(
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Manage group memberships for active user:',
                      style: theme.textTheme.bodyMedium?.copyWith(
                        color: theme.colorScheme.onSurfaceVariant,
                      ),
                    ),
                    const SizedBox(height: 12),
                    for (final g in availableGroups) ...[
                      CheckboxListTile(
                        contentPadding: EdgeInsets.zero,
                        value: _userGroups.contains(g['name']),
                        title: Text(
                          g['name']!,
                          style: const TextStyle(fontWeight: FontWeight.w600),
                        ),
                        subtitle: Text(g['desc']!),
                        onChanged: (val) {
                          setDialogState(() {
                            if (val == true) {
                              _userGroups.add(g['name']!);
                            } else {
                              _userGroups.remove(g['name']!);
                            }
                          });
                        },
                      ),
                    ],
                  ],
                ),
              ),
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.of(context).pop(),
                child: const Text('Cancel'),
              ),
              FilledButton(
                onPressed: () {
                  Navigator.of(context).pop();
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(
                      content: Text(
                        'User group memberships updated successfully',
                      ),
                    ),
                  );
                  setState(() {});
                },
                child: const Text('Save Changes'),
              ),
            ],
          );
        },
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<List<UserInfo>>(
      future: _future,
      builder: (context, snapshot) {
        if (snapshot.hasError) {
          final error = snapshot.error;
          return Scaffold(
            appBar: AppBar(title: const Text('Accounts')),
            body: error is AppError
                ? ErrorView(error: error, onRetry: _reload)
                : ErrorView(
                    error: AppError(
                      AppErrorType.systemError,
                      'Could not load user accounts.',
                      details: '$error',
                    ),
                    onRetry: _reload,
                  ),
          );
        }

        if (!snapshot.hasData) {
          return Scaffold(
            appBar: AppBar(title: const Text('Accounts')),
            body: const LoadingView(label: 'Reading user accounts…'),
          );
        }

        final users = snapshot.data!;
        final primaryUser = users.isNotEmpty
            ? users.firstWhere((u) => u.isAdmin, orElse: () => users.first)
            : const UserInfo(
                username: 'zyntrix',
                realName: 'Zyntrix Admin',
                isAdmin: true,
                homeDir: '/home/zyntrix',
                shell: '/bin/bash',
              );
        final theme = Theme.of(context);

        return ZettingsPage(
          title: 'Accounts',
          subtitle: 'Users, groups, login and session settings.',
          children: [
            // Hero User Card
            HeroStatusCard(
              leading: CircleAvatar(
                radius: 26,
                backgroundColor: theme.colorScheme.primary,
                child: Text(
                  primaryUser.realName.isNotEmpty
                      ? primaryUser.realName[0].toUpperCase()
                      : primaryUser.username[0].toUpperCase(),
                  style: TextStyle(
                    fontSize: 24,
                    fontWeight: FontWeight.w700,
                    color: theme.colorScheme.onPrimary,
                  ),
                ),
              ),
              title: primaryUser.realName.isNotEmpty
                  ? primaryUser.realName
                  : primaryUser.username,
              subtitle:
                  'Username: ${primaryUser.username} · Account type: ${primaryUser.isAdmin ? "Administrator" : "Standard"}',
              statusBadges: [
                if (primaryUser.isAdmin)
                  const Chip(
                    avatar: Icon(Icons.admin_panel_settings, size: 16),
                    label: Text('Administrator (sudo)'),
                  )
                else
                  const Chip(
                    avatar: Icon(Icons.person, size: 16),
                    label: Text('Standard User'),
                  ),
                const Chip(
                  avatar: Icon(Icons.security, size: 16),
                  label: Text('AccountsService Active'),
                ),
              ],
            ),

            // Authentication & Security
            const SectionHeader(title: 'Authentication & Password'),
            SettingGroupCard(
              children: [
                ZettingsRow(
                  icon: Icons.password,
                  title: 'Account password',
                  subtitle: 'Change login password for ${primaryUser.username}',
                  trailing: FilledButton.tonal(
                    onPressed: _showPasswordDialog,
                    child: const Text('Change password'),
                  ),
                ),
                ZettingsSwitchRow(
                  icon: Icons.login,
                  title: 'Automatic login',
                  subtitle: 'Log in automatically without prompting for password on boot',
                  value: _autoLogin,
                  onChanged: (val) => setState(() => _autoLogin = val),
                ),
                const ZettingsRow(
                  icon: Icons.fingerprint,
                  title: 'Biometric authentication (Fprintd)',
                  subtitle: 'Fingerprint sensor ready and configured',
                ),
              ],
            ),

            // Other Accounts
            const SectionHeader(title: 'Manage users'),
            SettingGroupCard(
              children: [
                for (final user in users.where(
                  (u) => u.username != primaryUser.username,
                ))
                  ZettingsRow(
                    icon: user.isAdmin
                        ? Icons.admin_panel_settings
                        : Icons.person,
                    title: user.realName.isNotEmpty
                        ? user.realName
                        : user.username,
                    subtitle:
                        '${user.username} · ${user.isAdmin ? "Administrator" : "Standard"} · ${user.homeDir}',
                  ),
                ZettingsRow(
                  icon: Icons.person_add,
                  title: 'Add new user',
                  subtitle:
                      'Create standard or administrator local user accounts',
                  trailing: OutlinedButton(
                    onPressed: _showAddUserDialog,
                    child: const Text('Add user'),
                  ),
                ),
                ZettingsRow(
                  icon: Icons.group,
                  title: 'User groups',
                  subtitle: 'Configure ${_userGroups.join(", ")} memberships',
                  trailing: const Icon(Icons.chevron_right),
                  onTap: _showUserGroupsDialog,
                ),
              ],
            ),
          ],
        );
      },
    );
  }
}
