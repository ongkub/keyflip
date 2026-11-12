(function () {
  'use strict';

  document.documentElement.classList.add('has-js');

  function detectOS() {
    var platform = '';
    if (navigator.userAgentData && typeof navigator.userAgentData.platform === 'string') {
      platform = navigator.userAgentData.platform.toLowerCase();
    }
    var ua = navigator.userAgent || '';
    var isIOS = /iPhone|iPad|iPod/i.test(ua) || (platform === 'macos' && navigator.maxTouchPoints > 1);

    if (isIOS) return 'ios';
    if (platform.includes('mac') || (!platform && /Macintosh|Mac OS X/i.test(ua))) return 'mac';
    if (platform.includes('win') || (!platform && /Windows|Win32|Win64/i.test(ua))) return 'windows';
    return 'other';
  }

  function detectMobile() {
    var isTouch = window.matchMedia && window.matchMedia('(pointer: coarse)').matches;
    var isSmall = window.innerWidth <= 860;
    var isMobileUA = /Android|iPhone|iPad|iPod/i.test(navigator.userAgent || '');
    return (isTouch && isSmall) || isMobileUA;
  }

  function initDownloadPanels(detectedOs) {
    var grid = document.getElementById('download-grid');
    var macPanel = document.getElementById('download-mac');
    var winPanel = document.getElementById('download-win');
    if (!grid || !macPanel || !winPanel) return;

    if (detectedOs === 'windows') {
      grid.insertBefore(winPanel, macPanel);
      var winBadge = winPanel.querySelector('.download-os-match');
      if (winBadge) winBadge.hidden = false;
    } else if (detectedOs === 'mac') {
      var macBadge = macPanel.querySelector('.download-os-match');
      if (macBadge) macBadge.hidden = false;
    }
  }

  function initMobileNotice() {
    var notice = document.getElementById('mobile-notice');
    var copyBtn = document.getElementById('copy-link-btn');

    if (notice && detectMobile()) {
      notice.hidden = false;
    }

    if (copyBtn) {
      copyBtn.addEventListener('click', async function () {
        var url = 'https://keyflip.sparkth.io/download/';
        var copied = false;

        if (navigator.clipboard && navigator.clipboard.writeText) {
          try {
            await navigator.clipboard.writeText(url);
            copied = true;
          } catch (_) {
            copied = false;
          }
        }

        if (!copied) {
          var ta = document.createElement('textarea');
          ta.value = url;
          ta.style.position = 'fixed';
          ta.style.opacity = '0';
          document.body.appendChild(ta);
          ta.select();
          try {
            copied = document.execCommand('copy');
          } catch (_) {
            copied = false;
          }
          document.body.removeChild(ta);
        }

        if (copied) {
          var originalText = copyBtn.textContent;
          copyBtn.textContent = 'คัดลอกแล้ว';
          copyBtn.disabled = true;
          setTimeout(function () {
            copyBtn.textContent = originalText;
            copyBtn.disabled = false;
          }, 2000);
        }
      });
    }
  }

  function initTabs(detectedOs) {
    var tablist = document.querySelector('.os-tablist');
    if (!tablist) return;

    var tabs = Array.from(tablist.querySelectorAll('[role="tab"]'));

    function selectTab(tabToSelect, focus) {
      tabs.forEach(function (tab) {
        var isSelected = tab === tabToSelect;
        tab.setAttribute('aria-selected', isSelected ? 'true' : 'false');
        tab.tabIndex = isSelected ? 0 : -1;

        var panelId = tab.getAttribute('aria-controls');
        var panel = document.getElementById(panelId);
        if (panel) {
          if (isSelected) {
            panel.removeAttribute('hidden');
          } else {
            panel.setAttribute('hidden', '');
          }
        }
      });

      if (focus) {
        tabToSelect.focus();
      }
    }

    tablist.addEventListener('keydown', function (e) {
      var activeEl = document.activeElement;
      var index = tabs.indexOf(activeEl);
      if (index === -1) return;

      var nextIndex = -1;
      if (e.key === 'ArrowRight' || e.key === 'ArrowDown') {
        nextIndex = (index + 1) % tabs.length;
      } else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {
        nextIndex = (index - 1 + tabs.length) % tabs.length;
      } else if (e.key === 'Home') {
        nextIndex = 0;
      } else if (e.key === 'End') {
        nextIndex = tabs.length - 1;
      }

      if (nextIndex !== -1) {
        e.preventDefault();
        selectTab(tabs[nextIndex], true);
      }
    });

    tabs.forEach(function (tab) {
      tab.addEventListener('click', function () {
        selectTab(tab, false);
      });
    });

    var initialId = detectedOs === 'windows' ? 'tab-win' : 'tab-mac';
    var initialTab = document.getElementById(initialId) || tabs[0];
    selectTab(initialTab, false);
  }

  // The download buttons open the install page for that OS, which starts the download itself, so people learn why
  // the first-open warning appears before they meet it. Without JS (or with a modifier key) the link downloads directly.
  function initInstallRedirect() {
    var pages = { 'download-mac': '/download/mac/', 'download-win': '/download/windows/' };
    Object.keys(pages).forEach(function (panelId) {
      var panel = document.getElementById(panelId);
      var button = panel && panel.querySelector('a.btn-primary[download]');
      if (!button) return;
      button.addEventListener('click', function (e) {
        if (e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
        e.preventDefault();
        window.location.href = pages[panelId] + '?start=1';
      });
    });
  }

  // On an install page opened from a download button (?start=1): start the download once, then drop the flag
  // so a reload or a shared link does not download again.
  function initAutoDownload() {
    var link = document.getElementById('download-fallback');
    if (!link || !/[?&]start=1(&|$)/.test(window.location.search)) return;
    // The page reads as a plain install guide; only now does it say the download is starting.
    Array.prototype.forEach.call(document.querySelectorAll('#download-status [data-active]'), function (el) {
      el.textContent = el.getAttribute('data-active');
    });
    if (window.history && window.history.replaceState) {
      window.history.replaceState(null, '', window.location.pathname + window.location.hash);
    }
    setTimeout(function () {
      window.location.href = link.href;
    }, 600);
  }

  function init() {
    var detectedOs = detectOS();
    initDownloadPanels(detectedOs);
    initMobileNotice();
    initTabs(detectedOs);
    initInstallRedirect();
    initAutoDownload();
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
